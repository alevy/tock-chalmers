## Adding a New Capsule to the Kernel

- [Intro](README.md)
- [Getting started with Tock](environment.md)
- [Write an environment sensing BLE application](application.md)
- Add a new capsule to the kernel

The goal of this part of the course is to give you a little bit of experience
with the Tock kernel and writing code for it. By the end, you'll
have written a new capsule that reads a light sensor and outputs its readings
over the serial port.

During this you will:

1. Learn how Tock uses Rust's memory safety to provide isolation for free
2. Read the Tock boot sequence, seeing how Tock loads processes
3. Learn about Tock's event-driven programming
4. Write a new capsule that periodically prints to the console

## 1. The Tock boot sequence (10m)

> The goal of this section is to give you an overview of how everything fits
> together. If you aren't familiar with Rust, don't stress too much about the
> syntax details, which can be a bit gnarly. Just try to read through the code
> here so you understand when things are getting called and how things are
> wired together.

Open `doc/courses/chalmers/exercises/board/src/main.rs` in your favorite editor.

This file defines a modified version of the Launchxl platform for this tutorial:
how it boots, what capsules it uses, and what system calls it supports for
userland applications. This version of the platform includes an extra "chalmers"
capsule, which you will implement in the rest of this tutorial.

Build this modified board now:

    cd doc/courses/chalmers/exercises/board && make

Rust will emit a preview of coming attractions as it warns about some of the
unused stubs we've included:

    ...
    warning: field is never used: `alarm`
      --> /home/alevy/hack/tock/doc/courses/chalmers/exercises/capsule/src/hello_world.rs:14:5
       |
    14 |     alarm: &'a A,
       |     ^^^^^^^^^^^^
       |
       = note: #[warn(dead_code)] on by default
    ...


### 1.1 How is everything organized?

Find the declaration of `struct Launchxl`.
This declares the structure representing the platform. It has many fields,
all of which are capsules. These are the capsules that make up the Launchxl
platform. For the most part, these map directly to hardware peripherals,
but there are exceptions such as `IPC` (inter-process communication).
In this tutorial, you'll be using the first two capsules, `console` and
`chalmers`.

Recall the discussion about how everything in the kernel is statically
allocated? We can see that here. Every field in `struct Launchxl` is a reference to
an object with a static lifetime.

As we walk through the rest of the boot process, we will construct a `Launchxl`
structure. Once everything is set up, the board passes the constructed `hail`
to `kernel::main` and the kernel is off to the races.

### 3.2 How are capsules created?

Scroll down a bit to line 111 to find the `reset_handler`. This is the first
function that's called on boot. It first has to set up a few things for the
underlying MCU (the `cc26x2`). Around line 168, we create and initialize the
system console capsule, which is what turns prints into bytes sent to the USB
serial port:

```rust
let console = static_init!(
    capsules::console::Console<cc26x2::uart::UART>,
    capsules::console::Console::new(
        &cc26x2::uart::UART0,
        115200,
        &mut capsules::console::WRITE_BUF,
        &mut capsules::console::READ_BUF,
        kernel::Grant::create()
    )
);
kernel::hil::uart::UART::set_client(&cc26x2::uart::UART0, console);
```

The `static_init!` macro allocates a static variable with a call to
`new`. The first parameter is the type, the second is the expression
to produce an instance of the type. This call creates a `Console` that
uses serial port 0 (`UART0`) at 115200 bits per second. The capsule needs
a mutable buffer to handle outgoing messages, and that is passed in here as
well. For convenience the actual buffer is defined in the console's source file,
but it could be defined here in the `main.rs` file instead. The capsule also
needs to be able to store per-application state, and the mechanism to allow that
(the `Container`) is also setup. Finally, once the `console` object exists,
we setup the callback chain so that events triggered by the `USART0` hardware
are correctly passed to the `console` object.


### 3.4 Let's make a Launchxl object (including your new capsule)!

After initializing the console, `reset_handler` creates all of the
other capsules that are needed by the Launchxl platform. If you look around
line 274, it initializes an instance of the `HelloWorld` capsule:

```rust
/* 1 */    let hello_alarm = static_init!(
                capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
                capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm));
/* 2 */    let hello_world = static_init!(chalmers::HelloWorld<'static, capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>>,
                chalmers::HelloWorld::new(hello_alarm));
/* 3 */    hello_alarm.set_client(hello_world);
/* 4 */    hello_world.start();
```

This code has four steps:

1. It creates a software alarm, which your `chalmers` capsule will use to
receive callbacks when time has passed.

2. It instantiates an `HelloWorld` object.
   - Recall that the first parameter to `static_init!` is the type, and the
     second is the instantiating function. The generic type `chalmers::HelloWorld`
     has two parameters:
       - a lifetime: `'static`
       - the type of its software alarm: `VirtualMuxAlarm<'static, sam4l::ast::Ast>`).
   - It's instantiated with a call to `new` that takes one parameters: a
     reference to the software alarm (`chalmers_virtual_alarm`).

3. It sets the client (the struct that receives callbacks) of the
software alarm to be the `chalmers` structure.

4. Finally, it runs the main capsule function, `start`.

After everything is wired together, the picture looks something like this:

Finally, at the very end, the kernel's main loop begins.

## 4. Create a "Hello World" capsule (15m)

Let's start by making sure there nothing installed on the board:

```bash
$ make erase
```

Now that you've seen how Tock initializes and uses capsules, including your
`HelloWorld` capsule, you're going to fill in the code for `HelloWorld.` At the end of
this section, your capsule will sample the light sensor and print the results
as serial output. But you'll start with something simpler: printing
"Hello Kernel" to the debug console once on boot.

Open the capsule `doc/courses/chalmers/exercises/capsule/src/hello_world.rs`. The kernel
boot sequence already includes this capsule, but its code is empty. Go to the
`start` method in the file, it looks like;

```rust
fn start(&self) -> ReturnCode {
```

Eventually, the `start` method will set an alarm for printing periodically, but
for now, you'll just print "Hello Kernel" to the debug console and return.  So
insert this line into the `start` method:

```rust
debug!("Hello Kernel");
```

Now compile and program your new kernel:

```bash
$ cd doc/courses/chalmers/exercises/board
$ make program
    [ ... several warnings here ... ]
$ tockloader listen
No device name specified. Using default "tock"
Listening for serial output.
Hello Kernel
```

## 5. Extend your capsule to print every second (10m)

For your capsule to keep track of time, it depends on another capsule that
implements the Alarm trait—a Rust trait is a mechanism for defining interfaces.
In Tock, an Alarm is a free running, wrap-around counter that can issue a
callback when the counter reaches a certain value.

The [time Hardware Interface Layer (HIL)](https://docs-tockosorg.netlify.com/kernel/hil/time/index.html)
defines several traits: `Alarm`, `Client`, and `Frequency`.

You'll ask `Alarm` when `now` is, and then `set_alarm` for a little bit in the
future. When the alarm triggers, it will call the `fired` callback as
specified by the `time::Client` trait.

### 5.1 When is now, when is one second from now?

First things first, lets figure out when `now` is:

```rust
let now = self.alarm.now();
```

Great! But.. what is `now`? Seconds since the epoch? Nanoseconds from boot?
If we examine [the HIL definition](https://docs-tockosorg.netlify.com/kernel/hil/time/trait.alarm#tymethod.now),
`now` is "current time in hardware clock units."

This is where the type of the Alarm (`A: Alarm + 'a`) in the definition of
`HelloWorld` comes into play. The type defines the time units.  So, calling
`<A::Frequency>::frequency()` will return the number of counter ticks in one
second. Which means we could do:

```rust
let one_second_from_now = now + <A::Frequency>::frequency(); // danger!
```

Unfortunately, things aren't quite that easy.  If you've ever dealt with the
hazards of mixing signed and unsigned numbers in C, implicit type conversions,
and the edge cases that can occur especially when you want to handle wraparound
correctly, you're probably a little nervous.  If we do this addition
incorrectly, then the whole system could pause for an almost entire cycle of
the 32-bit counter. Thankfully, Rust provides a helper function to take care of
cases such as these, `wrapping_add`, which in practice compiles down to the
correct addition instruction:

```rust
let one_second_from_now = now.wrapping_add(<A::Frequency>::frequency());
```

### 5.2 Set the alarm

The [`set_alarm` interface](https://docs-tockosorg.netlify.com/kernel/hil/time/trait.alarm#tymethod.set_alarm) looks like this:

```rust
fn set_alarm(&self, tics: u32);
```

So putting everything together:

```rust
pub fn start(&self) {
    let now = self.alarm.now();
    let one_second_from_now = now.wrapping_add(<A::Frequency>::frequency());
    self.alarm.set_alarm(one_second_from_now);
    debug!{"It's currently {} and we set an alarm for {}.", now, one_second_from_now};
}
```

    It's currently 323012278 and we set an alarm for 323028278.


### 5.3 Handle the `fired` callback

Currently, our `fired` callback isn't doing anything. Modify the `fired`
method to print every time it fires and then setup a new alarm so that
the callback will keep triggering every second:

    It's currently 326405717 and we set an alarm for 326421717.
    It's now 326421717.
    It's now 326437718.
    It's now 326453718.
    ...

