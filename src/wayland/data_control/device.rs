pub use super::server::zwlr_data_control_device_v1::{Request, ZwlrDataControlDeviceV1 as Device};

use std::cell::RefCell;

use tracing::debug;
use wayland_server::{protocol::wl_seat::WlSeat, Client, DataInit, Dispatch, DisplayHandle, Resource};

use crate::{
    input::{Seat, SeatHandler},
    wayland::seat::WaylandFocus,
};

use super::{Handler, SeatData, Selection, State};

#[doc(hidden)]
#[derive(Debug)]
pub struct Data {
    pub(crate) wl_seat: WlSeat,
}

impl<D> Dispatch<Device, Data, D> for State<D>
where
    D: Dispatch<Device, Data>,
    D: Handler,
    D: SeatHandler,
    <D as SeatHandler>::KeyboardFocus: WaylandFocus,
    D: 'static,
{
    fn request(
        handler: &mut D,
        client: &Client,
        resource: &Device,
        request: Request,
        data: &Data,
        dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        if let Some(seat) = Seat::<D>::from_resource(&data.wl_seat) {
            match request {
                Request::SetSelection { source, .. } => {
                    if let Some(keyboard) = seat.get_keyboard() {
                        if keyboard.client_of_object_has_focus(&resource.id()) {
                            let seat_data = seat.user_data().get::<RefCell<SeatData>>().unwrap();

                            Handler::new_selection(handler, source.clone());
                            // The client has kbd focus, it can set the selection
                            seat_data.borrow_mut().set_selection::<D>(
                                dh,
                                source.map(Selection::Client).unwrap_or(Selection::Empty),
                            );
                            return;
                        }
                    }
                    debug!(
                        client = ?client,
                        "denying setting selection by a non-focused client"
                    );
                }
                Request::Destroy => {
                    // Clean up the known devices
                    seat.user_data()
                        .get::<RefCell<SeatData>>()
                        .unwrap()
                        .borrow_mut()
                        .retain_devices(|ndd| ndd != resource)
                }
                _ => unreachable!(),
            }
        }
    }
}
