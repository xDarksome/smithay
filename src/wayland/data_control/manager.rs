pub use super::server::zwlr_data_control_manager_v1::{Request, ZwlrDataControlManagerV1 as Manager};

use std::cell::RefCell;

use tracing::error;
use wayland_server::{Dispatch, DisplayHandle, GlobalDispatch};

use crate::input::{Seat, SeatHandler};

use super::{Handler, SeatData, State};

use super::{device, source, Device, Source};

impl<D> GlobalDispatch<Manager, (), D> for State<D>
where
    D: SeatHandler + GlobalDispatch<Manager, ()>,
    D: Dispatch<Manager, ()>,
    D: Dispatch<Source, source::Data>,
    D: Dispatch<Device, device::Data>,
    D: Handler,
    D: 'static,
{
    fn bind(
        _state: &mut D,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        resource: wayland_server::New<Manager>,
        _global_data: &(),
        data_init: &mut wayland_server::DataInit<'_, D>,
    ) {
        data_init.init(resource, ());
    }
}

impl<D> Dispatch<Manager, (), D> for State<D>
where
    D: Dispatch<Manager, ()>,
    D: Dispatch<Source, source::Data>,
    D: Dispatch<Device, device::Data>,
    D: Handler,
    D: SeatHandler,
    D: 'static,
{
    fn request(
        _state: &mut D,
        client: &wayland_server::Client,
        _resource: &Manager,
        request: Request,
        _data: &(),
        _dhandle: &DisplayHandle,
        data_init: &mut wayland_server::DataInit<'_, D>,
    ) {
        match request {
            Request::CreateDataSource { id } => {
                data_init.init(id, source::Data::new());
            }
            Request::GetDataDevice { id, seat: wl_seat } => match Seat::<D>::from_resource(&wl_seat) {
                Some(seat) => {
                    seat.user_data()
                        .insert_if_missing(|| RefCell::new(SeatData::new()));

                    let device = data_init.init(id, device::Data { wl_seat });

                    let seat_data = seat.user_data().get::<RefCell<SeatData>>().unwrap();
                    seat_data.borrow_mut().add_device(device);
                }
                None => {
                    error!(
                        primary_selection_device = ?id,
                        client = ?client,
                        "Unmanaged seat given to a primary selection device."
                    );
                }
            },
            Request::Destroy => {}
            _ => unreachable!(),
        }
    }
}
