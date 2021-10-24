//! Common traits for input backends to receive input from.

use std::{error::Error, path::PathBuf};

mod tablet;

pub use tablet::{
    ProximityState, TabletToolAxisEvent, TabletToolButtonEvent, TabletToolCapabilitys, TabletToolDescriptor,
    TabletToolEvent, TabletToolProximityEvent, TabletToolTipEvent, TabletToolTipState, TabletToolType,
};

use crate::utils::{Logical, Point, Raw, Size};

/// Trait for generic functions every input device does provide
pub trait Device: PartialEq + Eq + std::hash::Hash {
    /// Unique id of a single device at a point in time.
    ///
    /// Note: This means ids may be re-used by the backend for later devices.
    fn id(&self) -> String;
    /// Human-readable name of the device
    fn name(&self) -> String;
    /// Test if this device has a specific capability
    fn has_capability(&self, capability: DeviceCapability) -> bool;

    /// Returns device USB (product,vendor) id
    fn usb_id(&self) -> Option<(u32, u32)>;

    /// Returns the syspath of the device.
    ///
    /// The path is an absolute path and includes the sys mount point.
    fn syspath(&self) -> Option<PathBuf>;
}

/// Set of input types a device may provide
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)] // self explainatory
pub enum DeviceCapability {
    Keyboard,
    Pointer,
    Touch,
    TabletTool,
    TabletPad,
    Gesture,
    Switch,
}

/// Trait for generic functions every input event does provide
pub trait Event<B: InputBackend> {
    /// Returns an upward counting variable useful for event ordering.
    ///
    /// Makes no guarantees about actual time passed between events.
    // # TODO:
    // - check if events can even arrive out of order.
    // - Make stronger time guarantees, if possible
    fn time(&self) -> u32;
    /// Returns the device, that generated this event
    fn device(&self) -> B::Device;
}

/// Used to mark events never emitted by an [`InputBackend`] implementation.
///
/// Implements all event types and can be used in place for any [`Event`] type,
/// that is not used by an [`InputBackend`] implementation. Initialization is not
/// possible, making accidental use impossible and enabling a lot of possible
/// compiler optimizations.
#[derive(Debug)]
pub enum UnusedEvent {}

impl<B: InputBackend> Event<B> for UnusedEvent {
    fn time(&self) -> u32 {
        match *self {}
    }

    fn device(&self) -> B::Device {
        match *self {}
    }
}

/// State of key on a keyboard. Either pressed or released
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyState {
    /// Key is released
    Released,
    /// Key is pressed
    Pressed,
}

/// Trait for keyboard event
pub trait KeyboardKeyEvent<B: InputBackend>: Event<B> {
    /// Returns the numerical button code of the keyboard button.
    ///
    /// The value will correspond to one `KEY_` constants from  the Linux [input event codes] inside
    /// `input-event-codes.h`.
    ///
    /// [input event codes]: https://gitlab.freedesktop.org/libinput/libinput/-/blob/main/include/linux/linux/input-event-codes.h
    fn key_code(&self) -> u32;

    /// State of the key
    fn state(&self) -> KeyState;

    /// Total number of keys pressed on all devices on the associated [`Seat`](crate::wayland::seat::Seat)
    fn count(&self) -> u32;
}

impl<B: InputBackend> KeyboardKeyEvent<B> for UnusedEvent {
    fn key_code(&self) -> u32 {
        match *self {}
    }

    fn state(&self) -> KeyState {
        match *self {}
    }

    fn count(&self) -> u32 {
        match *self {}
    }
}

/// A particular mouse button
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button
    Middle,
    /// Right mouse button
    Right,
    /// Other mouse button with index
    Other(u8),
}

/// State of a button on a pointer device, like mouse or tablet tool. Either pressed or released
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ButtonState {
    /// Button is released
    Released,
    /// Button is pressed
    Pressed,
}

/// Common methods pointer event generated by pressed buttons do implement
pub trait PointerButtonEvent<B: InputBackend>: Event<B> {
    /// Pressed button of the event
    fn button(&self) -> MouseButton;
    /// State of the button
    fn state(&self) -> ButtonState;
}

impl<B: InputBackend> PointerButtonEvent<B> for UnusedEvent {
    fn button(&self) -> MouseButton {
        match *self {}
    }

    fn state(&self) -> ButtonState {
        match *self {}
    }
}

/// Axis when scrolling
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Axis {
    /// Vertical axis
    Vertical,
    /// Horizontal axis
    Horizontal,
}

/// Source of an axis when scrolling
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AxisSource {
    /// Finger. Mostly used for trackpads.
    ///
    /// Guarantees that a scroll sequence is terminated with a scroll value of 0.
    /// A caller may use this information to decide on whether kinetic scrolling should
    /// be triggered on this scroll sequence.
    ///
    /// The coordinate system is identical to the
    /// cursor movement, i.e. a scroll value of 1 represents the equivalent relative
    /// motion of 1.
    Finger,
    /// Continuous scrolling device. Almost identical to [`Finger`](AxisSource::Finger)
    ///
    /// No terminating event is guaranteed (though it may happen).
    ///
    /// The coordinate system is identical to
    /// the cursor movement, i.e. a scroll value of 1 represents the equivalent relative
    /// motion of 1.
    Continuous,
    /// Scroll wheel.
    ///
    /// No terminating event is guaranteed (though it may happen). Scrolling is in
    /// discrete steps. It is up to the caller how to interpret such different step sizes.
    Wheel,
    /// Scrolling through tilting the scroll wheel.
    ///
    /// No terminating event is guaranteed (though it may happen). Scrolling is in
    /// discrete steps. It is up to the caller how to interpret such different step sizes.
    WheelTilt,
}

/// Trait for pointer events generated by scrolling on an axis.
pub trait PointerAxisEvent<B: InputBackend>: Event<B> {
    /// Amount of scrolling in pixels on the given [`Axis`].
    ///
    /// Guaranteed to be `Some` when source returns either [`AxisSource::Finger`] or [`AxisSource::Continuous`].
    fn amount(&self, axis: Axis) -> Option<f64>;

    /// Amount of scrolling in discrete steps on the given [`Axis`].
    ///
    /// Guaranteed to be `Some` when source returns either [`AxisSource::Wheel`] or [`AxisSource::WheelTilt`].
    fn amount_discrete(&self, axis: Axis) -> Option<f64>;

    /// Source of the scroll event.
    fn source(&self) -> AxisSource;
}

impl<B: InputBackend> PointerAxisEvent<B> for UnusedEvent {
    fn amount(&self, _axis: Axis) -> Option<f64> {
        match *self {}
    }

    fn amount_discrete(&self, _axis: Axis) -> Option<f64> {
        match *self {}
    }

    fn source(&self) -> AxisSource {
        match *self {}
    }
}

/// Trait for pointer events generated by relative device movement.
pub trait PointerMotionEvent<B: InputBackend>: Event<B> {
    /// Delta between the last and new pointer device position interpreted as pixel movement
    fn delta(&self) -> Point<f64, Logical> {
        (self.delta_x(), self.delta_y()).into()
    }

    /// Delta on the x axis between the last and new pointer device position interpreted as pixel movement
    fn delta_x(&self) -> f64;
    /// Delta on the y axis between the last and new pointer device position interpreted as pixel movement
    fn delta_y(&self) -> f64;
}

impl<B: InputBackend> PointerMotionEvent<B> for UnusedEvent {
    fn delta_x(&self) -> f64 {
        match *self {}
    }

    fn delta_y(&self) -> f64 {
        match *self {}
    }
}

/// Trait for pointer events generated by absolute device positioning.
pub trait PointerMotionAbsoluteEvent<B: InputBackend>: Event<B> {
    /// Device position in it's original coordinate space.
    ///
    /// The format is defined by the backend implementation.
    fn position(&self) -> Point<f64, Raw> {
        (self.x(), self.y()).into()
    }

    /// Device x position in it's original coordinate space.
    ///
    /// The format is defined by the backend implementation.
    fn x(&self) -> f64;

    /// Device y position in it's original coordinate space.
    ///
    /// The format is defined by the backend implementation.
    fn y(&self) -> f64;

    /// Device position converted to the targets coordinate space.
    /// E.g. the focused output's resolution.
    fn position_transformed(&self, coordinate_space: Size<i32, Logical>) -> Point<f64, Logical> {
        (
            self.x_transformed(coordinate_space.w),
            self.y_transformed(coordinate_space.h),
        )
            .into()
    }

    /// Device x position converted to the targets coordinate space's width.
    /// E.g. the focused output's width.
    fn x_transformed(&self, width: i32) -> f64;

    /// Device y position converted to the targets coordinate space's height.
    /// E.g. the focused output's height.
    fn y_transformed(&self, height: i32) -> f64;
}

impl<B: InputBackend> PointerMotionAbsoluteEvent<B> for UnusedEvent {
    fn x(&self) -> f64 {
        match *self {}
    }

    fn y(&self) -> f64 {
        match *self {}
    }

    fn x_transformed(&self, _width: i32) -> f64 {
        match *self {}
    }

    fn y_transformed(&self, _height: i32) -> f64 {
        match *self {}
    }
}

/// Slot of a different touch event.
///
/// Touch events are grouped by slots, usually to identify different
/// fingers on a multi-touch enabled input device. Events should only
/// be interpreted in the context of other events on the same slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TouchSlot {
    id: u64,
}

#[cfg(any(feature = "backend_winit", feature = "backend_libinput"))]
impl TouchSlot {
    pub(crate) fn new(id: u64) -> Self {
        TouchSlot { id }
    }
}

/// Trait for touch events starting at a given position.
pub trait TouchDownEvent<B: InputBackend>: Event<B> {
    /// [`TouchSlot`], if the device has multi-touch capabilities
    fn slot(&self) -> Option<TouchSlot>;

    /// Touch position in the device's native coordinate space
    ///
    /// The actual format is defined by the implementation.
    fn position(&self) -> Point<f64, Raw> {
        (self.x(), self.y()).into()
    }

    /// Touch position converted into the target coordinate space.
    /// E.g. the focused output's resolution.
    fn position_transformed(&self, coordinate_space: Size<i32, Logical>) -> Point<f64, Logical> {
        (
            self.x_transformed(coordinate_space.w),
            self.y_transformed(coordinate_space.h),
        )
            .into()
    }

    /// Touch event's x-coordinate in the device's native coordinate space
    ///
    /// The actual format is defined by the implementation.
    fn x(&self) -> f64;

    /// Touch event's x-coordinate in the device's native coordinate space
    ///
    /// The actual format is defined by the implementation.
    fn y(&self) -> f64;

    /// Touch event's x position converted to the targets coordinate space's width.
    /// E.g. the focused output's width.
    fn x_transformed(&self, width: i32) -> f64;

    /// Touch event's y position converted to the targets coordinate space's width.
    /// E.g. the focused output's width.
    fn y_transformed(&self, height: i32) -> f64;
}

impl<B: InputBackend> TouchDownEvent<B> for UnusedEvent {
    fn slot(&self) -> Option<TouchSlot> {
        match *self {}
    }

    fn x(&self) -> f64 {
        match *self {}
    }

    fn y(&self) -> f64 {
        match *self {}
    }

    fn x_transformed(&self, _width: i32) -> f64 {
        match *self {}
    }

    fn y_transformed(&self, _height: i32) -> f64 {
        match *self {}
    }
}

/// Trait for touch events regarding movement on the screen
pub trait TouchMotionEvent<B: InputBackend>: Event<B> {
    /// [`TouchSlot`], if the device has multi-touch capabilities
    fn slot(&self) -> Option<TouchSlot>;

    /// Touch position in the device's native coordinate space
    ///
    /// The actual format is defined by the implementation.
    fn position(&self) -> Point<f64, Raw> {
        (self.x(), self.y()).into()
    }

    /// Touch position converted into the target coordinate space.
    /// E.g. the focused output's resolution.
    fn position_transformed(&self, coordinate_space: Size<i32, Logical>) -> Point<f64, Logical> {
        (
            self.x_transformed(coordinate_space.w),
            self.y_transformed(coordinate_space.h),
        )
            .into()
    }

    /// Touch event's x-coordinate in the device's native coordinate space
    ///
    /// The actual format is defined by the implementation.
    fn x(&self) -> f64;

    /// Touch event's x-coordinate in the device's native coordinate space
    ///
    /// The actual format is defined by the implementation.
    fn y(&self) -> f64;

    /// Touch event's x position converted to the targets coordinate space's width.
    /// E.g. the focused output's width.
    fn x_transformed(&self, width: i32) -> f64;

    /// Touch event's y position converted to the targets coordinate space's width.
    /// E.g. the focused output's width.
    fn y_transformed(&self, height: i32) -> f64;
}

impl<B: InputBackend> TouchMotionEvent<B> for UnusedEvent {
    fn slot(&self) -> Option<TouchSlot> {
        match *self {}
    }

    fn x(&self) -> f64 {
        match *self {}
    }

    fn y(&self) -> f64 {
        match *self {}
    }

    fn x_transformed(&self, _width: i32) -> f64 {
        match *self {}
    }

    fn y_transformed(&self, _height: i32) -> f64 {
        match *self {}
    }
}

/// Trait for touch events finishing.
pub trait TouchUpEvent<B: InputBackend>: Event<B> {
    /// [`TouchSlot`], if the device has multi-touch capabilities
    fn slot(&self) -> Option<TouchSlot>;
}

impl<B: InputBackend> TouchUpEvent<B> for UnusedEvent {
    fn slot(&self) -> Option<TouchSlot> {
        match *self {}
    }
}

/// Trait for touch events canceling the chain
pub trait TouchCancelEvent<B: InputBackend>: Event<B> {
    /// [`TouchSlot`], if the device has multi-touch capabilities
    fn slot(&self) -> Option<TouchSlot>;
}

impl<B: InputBackend> TouchCancelEvent<B> for UnusedEvent {
    fn slot(&self) -> Option<TouchSlot> {
        match *self {}
    }
}

/// Trait for touch frame events
pub trait TouchFrameEvent<B: InputBackend>: Event<B> {}

impl<B: InputBackend> TouchFrameEvent<B> for UnusedEvent {}

/// Trait that describes objects providing a source of input events. All input backends
/// need to implement this and provide the same base guarantees about the precision of
/// given events.
pub trait InputBackend: Sized {
    /// Type representing errors that may be returned when processing events
    type EventError: Error;

    /// Type representing input devices
    type Device: Device;
    /// Type representing keyboard events
    type KeyboardKeyEvent: KeyboardKeyEvent<Self>;
    /// Type representing axis events on pointer devices
    type PointerAxisEvent: PointerAxisEvent<Self>;
    /// Type representing button events on pointer devices
    type PointerButtonEvent: PointerButtonEvent<Self>;
    /// Type representing motion events of pointer devices
    type PointerMotionEvent: PointerMotionEvent<Self>;
    /// Type representing motion events of pointer devices
    type PointerMotionAbsoluteEvent: PointerMotionAbsoluteEvent<Self>;
    /// Type representing touch events starting
    type TouchDownEvent: TouchDownEvent<Self>;
    /// Type representing touch events ending
    type TouchUpEvent: TouchUpEvent<Self>;
    /// Type representing touch events from moving
    type TouchMotionEvent: TouchMotionEvent<Self>;
    /// Type representing canceling of touch events
    type TouchCancelEvent: TouchCancelEvent<Self>;
    /// Type representing touch frame events
    type TouchFrameEvent: TouchFrameEvent<Self>;
    /// Type representing axis events on tablet devices
    type TabletToolAxisEvent: TabletToolAxisEvent<Self>;
    /// Type representing proximity events on tablet devices
    type TabletToolProximityEvent: TabletToolProximityEvent<Self>;
    /// Type representing tip events on tablet devices
    type TabletToolTipEvent: TabletToolTipEvent<Self>;
    /// Type representing button events on tablet tool devices
    type TabletToolButtonEvent: TabletToolButtonEvent<Self>;

    /// Special events that are custom to this backend
    type SpecialEvent;

    /// Processes new events and calls the provided callback.
    fn dispatch_new_events<F>(&mut self, callback: F) -> Result<(), Self::EventError>
    where
        F: FnMut(InputEvent<Self>);
}

/// Different events that can be generated by an input backend
#[derive(Debug)]
pub enum InputEvent<B: InputBackend> {
    /// An input device was connected
    DeviceAdded {
        /// The added device
        device: B::Device,
    },
    /// An input device was disconnected
    DeviceRemoved {
        /// The removed device
        device: B::Device,
    },
    /// A keyboard event occurred
    Keyboard {
        /// The keyboard event
        event: B::KeyboardKeyEvent,
    },
    /// A relative pointer motion occurred
    PointerMotion {
        /// The pointer motion event
        event: B::PointerMotionEvent,
    },
    /// An absolute pointer motion occurs
    PointerMotionAbsolute {
        /// The absolute pointer motion event
        event: B::PointerMotionAbsoluteEvent,
    },
    /// A pointer button was pressed or released
    PointerButton {
        /// The pointer button event
        event: B::PointerButtonEvent,
    },
    /// A pointer action occurred while scrolling on an axis
    PointerAxis {
        /// The pointer axis event
        event: B::PointerAxisEvent,
    },
    /// A new touchpoint appeared
    TouchDown {
        /// The touch down event
        event: B::TouchDownEvent,
    },
    /// A touchpoint moved
    TouchMotion {
        /// The touch motion event
        event: B::TouchMotionEvent,
    },
    /// A touchpoint was removed
    TouchUp {
        /// The touch up event
        event: B::TouchUpEvent,
    },
    /// A touch sequence was cancelled
    TouchCancel {
        /// The touch cancel event
        event: B::TouchCancelEvent,
    },
    /// A touch frame was emitted
    ///
    /// A set of two events received on the same seat between two frames should
    /// be interpreted as an atomic event.
    TouchFrame {
        /// The touch frame event
        event: B::TouchFrameEvent,
    },

    /// A tablet tool axis was emitted
    TabletToolAxis {
        /// The tablet tool axis event
        event: B::TabletToolAxisEvent,
    },

    /// A tablet tool proximity was emitted
    TabletToolProximity {
        /// The tablet tool proximity  event
        event: B::TabletToolProximityEvent,
    },

    /// A tablet tool tip event was emitted
    TabletToolTip {
        /// The tablet tool axis event
        event: B::TabletToolTipEvent,
    },

    /// A tablet tool button was pressed or released
    TabletToolButton {
        /// The pointer button event
        event: B::TabletToolButtonEvent,
    },

    /// Special event specific of this backend
    Special(B::SpecialEvent),
}
