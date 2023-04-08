pub use super::server::zwlr_data_control_source_v1::{Request, ZwlrDataControlSourceV1 as Source};

use std::sync::Mutex;

use wayland_server::{
    backend::{ClientId, ObjectId},
    Dispatch, DisplayHandle, Resource,
};

use crate::{
    input::SeatHandler,
    utils::{alive_tracker::AliveTracker, IsAlive},
};

use super::{Handler, State};

/// The metadata describing a data source
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    /// The MIME types supported by this source
    pub mime_types: Vec<String>,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct Data {
    inner: Mutex<Metadata>,
    alive_tracker: AliveTracker,
}

impl Data {
    pub(super) fn new() -> Self {
        Self {
            inner: Default::default(),
            alive_tracker: Default::default(),
        }
    }
}

impl<D> Dispatch<Source, Data, D> for State<D>
where
    D: SeatHandler + Dispatch<Source, Data>,
    D: Handler,
    D: 'static,
{
    fn request(
        _state: &mut D,
        _client: &wayland_server::Client,
        _resource: &Source,
        request: Request,
        data: &Data,
        _dhandle: &DisplayHandle,
        _data_init: &mut wayland_server::DataInit<'_, D>,
    ) {
        // let _primary_selection_state = state.primary_selection_state();
        let mut data = data.inner.lock().unwrap();

        match request {
            Request::Offer { mime_type } => {
                data.mime_types.push(mime_type);
            }
            Request::Destroy => {}
            _ => unreachable!(),
        }
    }

    fn destroyed(_state: &mut D, _client: ClientId, _resource: ObjectId, data: &Data) {
        data.alive_tracker.destroy_notify();
    }
}

impl IsAlive for Source {
    fn alive(&self) -> bool {
        let data: &Data = self.data().unwrap();
        data.alive_tracker.alive()
    }
}

/// Access the metadata of a data source
pub fn with_source_metadata<T, F: FnOnce(&Metadata) -> T>(
    source: &Source,
    f: F,
) -> Result<T, crate::utils::UnmanagedResource> {
    match source.data::<Data>() {
        Some(data) => Ok(f(&data.inner.lock().unwrap())),
        None => Err(crate::utils::UnmanagedResource),
    }
}
