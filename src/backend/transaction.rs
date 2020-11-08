use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;

use crate::backend::{SoukPackage, SoukPackageAction, SoukTransactionState};

pub struct SoukTransactionPrivate {
    package: RefCell<Option<SoukPackage>>,
    action: RefCell<Option<SoukPackageAction>>,
    state: RefCell<Option<SoukTransactionState>>,
}

static PROPERTIES: [subclass::Property; 3] = [
    subclass::Property("package", |package| {
        glib::ParamSpec::object(
            package,
            "Package",
            "Package",
            SoukPackage::static_type(),
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("action", |action| {
        glib::ParamSpec::enum_(
            action,
            "Action",
            "Action",
            SoukPackageAction::static_type(),
            SoukPackageAction::default() as i32,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("state", |state| {
        glib::ParamSpec::object(
            state,
            "State",
            "State",
            SoukTransactionState::static_type(),
            glib::ParamFlags::READWRITE,
        )
    }),
];

impl ObjectSubclass for SoukTransactionPrivate {
    const NAME: &'static str = "SoukTransaction";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        SoukTransactionPrivate {
            package: RefCell::default(),
            action: RefCell::default(),
            state: RefCell::default(),
        }
    }
}

impl ObjectImpl for SoukTransactionPrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => Ok(self.package.borrow().to_value()),
            subclass::Property("action", ..) => {
                Ok(self.action.borrow().as_ref().unwrap().to_value())
            }
            subclass::Property("state", ..) => Ok(self.state.borrow().to_value()),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("state", ..) => {
                let state = value.get().unwrap();
                *self.state.borrow_mut() = state;
            }
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukTransaction(
        Object<subclass::simple::InstanceStruct<SoukTransactionPrivate>,
        subclass::simple::ClassStruct<SoukTransactionPrivate>>);

    match fn {
        get_type => || SoukTransactionPrivate::get_type().to_glib(),
    }
}

#[allow(dead_code)]
impl SoukTransaction {
    pub fn new(package: SoukPackage, action: SoukPackageAction) -> Self {
        let transaction = glib::Object::new(SoukTransaction::static_type(), &[])
            .unwrap()
            .downcast::<SoukTransaction>()
            .unwrap();

        let self_ = SoukTransactionPrivate::from_instance(&transaction);
        *self_.package.borrow_mut() = Some(package);
        *self_.action.borrow_mut() = Some(action);

        transaction
    }

    pub fn get_package(&self) -> SoukPackage {
        self.get_property("package")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_action(&self) -> SoukPackageAction {
        self.get_property("action").unwrap().get().unwrap().unwrap()
    }

    pub fn get_state(&self) -> SoukTransactionState {
        self.get_property("state").unwrap().get().unwrap().unwrap()
    }
}
