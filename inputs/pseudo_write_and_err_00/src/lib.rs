use std::marker::PhantomData;

// needed by Config::Origin https://substrate.dev/rustdocs/v3.0.0/frame_system/enum.RawOrigin.html
pub enum RawOrigin<AccountId> {
    Root,
    Signed(AccountId),
    None,
}

pub struct DispatchErrorWithPostInfo<Info> 
where
Info: Eq + PartialEq + Clone + Copy, //+ Encode + Decode + Printable, 
{
    pub post_info: Info,
    pub error: DispatchError,
}

//type DispatchResultWithInfo<T> = Result<T, DispatchErrorWithPostInfo<T>>;

// https://substrate.dev/rustdocs/v3.0.0/sp_runtime/traits/trait.Dispatchable.html
pub trait Dispatchable {
    // type Origin;
    // type Config;
    // type Info;
    // type PostInfo: Eq + PartialEq + Clone + Copy; // + Encode + Decode + Printable;
    // fn dispatch(
    //         self, 
    //         origin: Self::Origin
    //         ) -> DispatchResultWithInfo<Self::PostInfo>;
}

pub struct CallImpl {}
impl Dispatchable for CallImpl {}

// from whole Config let's take only Origin for now - source: https://substrate.dev/rustdocs/v3.0.0/frame_system/pallet/trait.Config.html
pub trait OriginTrait: Sized {
        type Call;
        // rest <cut> for now , as we need this as dep. (rest is https://substrate.dev/rustdocs/v3.0.0/frame_support/traits/trait.OriginTrait.html )
}

#[derive(Clone)]
pub struct OriginImpl {}
//impl Into<Result<RawOrigin<Self::AccountId>, Self::Origin>> for OriginImpl {
impl Into<Result<RawOrigin<i32>, OriginImpl>> for OriginImpl {
    fn into(self) -> Result<RawOrigin<i32>, OriginImpl> {
        todo!()
    }
}
impl From<RawOrigin<i32>> for OriginImpl {
    fn from(_: RawOrigin<i32>) -> Self {
        todo!()
    }
}
impl OriginTrait for OriginImpl {
    type Call = CallImpl;
}

pub trait Config: 'static + Eq + Clone {
    type AccountId: Ord + Default; //Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;
    type Call: Dispatchable; // + Debug;
    type Origin: Into<Result<RawOrigin<Self::AccountId>, Self::Origin>> + From<RawOrigin<Self::AccountId>> + Clone + OriginTrait<Call = Self::Call>;
}

#[derive(Clone)]
pub struct ConfigImpl {}
impl Config for ConfigImpl {
    type AccountId = i32;
    type Call = CallImpl;
    type Origin = OriginImpl;
}

impl Eq for ConfigImpl {}
impl PartialEq for ConfigImpl {
    fn eq(&self, _other: &Self) -> bool { true }
}

// from https://substrate.dev/rustdocs/v3.0.0/frame_system/pallet_prelude/type.OriginFor.html
type OriginFor<T> = <T as Config>::Origin;

//struct OriginForImpl<T> {
//    phantomdata: PhantomData<T>,
//}D

//impl<T> <T>::Origin for OriginForImpl<T> {}

struct XYZ<T> {
    phantomdata: PhantomData<T>,
}

impl<T> XYZ<T> {
    fn put(_b: bool) { }
}

// based on https://substrate.dev/rustdocs/v3.0.0/frame_support/dispatch/type.DispatchResult.html
type DispatchResult = Result<(), DispatchError>;

// based on https://substrate.dev/rustdocs/v3.0.0/frame_support/pallet_prelude/enum.DispatchError.html
pub enum DispatchError {
    Other(&'static str),
        CannotLookup,
        BadOrigin,
        Module {
            index: u8,
           error: u8,
           message: Option<&'static str>,
        },
        ConsumerRemaining,
        NoProviders,
}

fn ensure_root<T: Config>(_origin: OriginFor<T>) -> DispatchResult {
    Ok(())
}

// Example from : https://github.com/paritytech/substrate/issues/8962#issuecomment-851923189
pub fn xyz_should_match<T: Config>(origin: OriginFor<T>) -> DispatchResult {
  XYZ::<T>::put(true);
  ensure_root::<T>(origin)?;
  Ok(())
  // this pattern is wrong because we could both change storage and return an error
}

// Example from : https://github.com/paritytech/substrate/issues/8962#issuecomment-851923189
pub fn xyz_should_not_match<T: Config>(origin: OriginFor<T>) -> DispatchResult {
    XYZ::<T>::put(true);
  let _ret = ensure_root::<T>(origin)?;
  Ok(())
  // this pattern is wrong because we could both change storage and return an error
}


#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
