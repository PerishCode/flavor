#![allow(dead_code)]

use crate::sfc::VueSfcDescriptor;

pub trait VueVisitor {
    fn visit_sfc(&mut self, _descriptor: &VueSfcDescriptor) {}
}
