use anyq_queue_api::{CompleteDeliveryRequest, RejectDeliveryRequest};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Default)]
pub struct Trace {
    pub stopped: Rc<Cell<bool>>,
    pub ticks: Rc<Cell<u32>>,
    pub renewals: Rc<Cell<u32>>,
    pub completions: Rc<RefCell<Vec<CompleteDeliveryRequest>>>,
    pub rejections: Rc<RefCell<Vec<RejectDeliveryRequest>>>,
}
