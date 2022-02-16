use std::pin::Pin;

use derive_more::{Add, AddAssign, Display, From, Into, Mul, MulAssign, Neg, Sub, SubAssign};
use fixed::{traits::ToFixed, types::I12F20};
use futures::Future;

// [-2048, 2048) range, ~1M fractional resolution
// wrapping because we always want to be able to construct a 2's complement difference
// for delta encoding
pub type Num = fixed::Wrapping<I12F20>;

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    Add,
    AddAssign,
    Display,
    From,
    Into,
    Mul,
    MulAssign,
    Neg,
    Sub,
    SubAssign,
    Serialize,
    Deserialize,
)]
#[display(fmt = "V2 {{ x: {}, y: {} }}", x, y)]
pub struct V2 {
    pub x: Num,
    pub y: Num,
}
impl V2 {
    pub fn new<X: ToFixed, Y: ToFixed>(x: X, y: Y) -> Self {
        V2 {
            x: Num::from_num(x),
            y: Num::from_num(y),
        }
    }
}

#[macro_export]
macro_rules! mk_v2 {
    ($x:expr, $y:expr) => {
        V2 {
            x: fixed::Wrapping(fixed_macro::fixed!($x: I12F20)),
            y: fixed::Wrapping(fixed_macro::fixed!($y: I12F20)),
        }
    };
}

// impl std::ops::Mul<Num> for V2 {
//     type Output = V2;

//     fn mul(self, scalar: Num) -> Self {
//         Self {
//             x: self.x * scalar,
//             y: self.y * scalar,
//         }
//     }
// }

pub type Zed = std::num::Wrapping<i8>;
pub const fn mk_zed(zed: i8) -> Zed {
    std::num::Wrapping(zed)
}

pub type R = Num;

// just like BoxFuture but not send, because this will
// be used in the single threaded browser environment
pub type SharedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

macro_rules! map_types {
    (|$param:ident| $body:block, ($($kinds:ty),*)) => {
        ($({
            type $param = $kinds;
            $body
        }),*)
    };
}

macro_rules! map_all_components {
    (|$param:ident| $body:block) => {
        map_types!(
            |$param| $body,
            (Position, Rotation, Velocity, Camera, Player, Input, Bullet, Health)
        )
    };
}

pub(crate) use map_all_components;
pub(crate) use map_types;

macro_rules! derive_components {
    ($($i:item)*) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            $i
        )*
    }
}

macro_rules! derive_math_components {

    ($($i:item)*) => {
        derive_components! {
            $(
                #[derive(derive_more::Add, derive_more::AddAssign, derive_more::Sub, derive_more::SubAssign, derive_more::Neg)]
                $i
            )*
        }
    }
}

// same as components for now
macro_rules! derive_delta {
    ($($i:item)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            $i
        )*
    }
}

pub(crate) use derive_components;
pub(crate) use derive_delta;
pub(crate) use derive_math_components;

use serde::{Deserialize, Serialize};
