// automatically generated by the FlatBuffers compiler, do not modify
// @generated
extern crate alloc;
extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};
use super::*;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem;
pub enum FunctionCallOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct FunctionCall<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for FunctionCall<'a> {
    type Inner = FunctionCall<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table::new(buf, loc),
        }
    }
}

impl<'a> FunctionCall<'a> {
    pub const VT_FUNCTION_NAME: flatbuffers::VOffsetT = 4;
    pub const VT_PARAMETERS: flatbuffers::VOffsetT = 6;
    pub const VT_FUNCTION_CALL_TYPE: flatbuffers::VOffsetT = 8;

    #[inline]
    pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        FunctionCall { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args FunctionCallArgs<'args>,
    ) -> flatbuffers::WIPOffset<FunctionCall<'bldr>> {
        let mut builder = FunctionCallBuilder::new(_fbb);
        if let Some(x) = args.parameters {
            builder.add_parameters(x);
        }
        if let Some(x) = args.function_name {
            builder.add_function_name(x);
        }
        builder.add_function_call_type(args.function_call_type);
        builder.finish()
    }

    #[inline]
    pub fn function_name(&self) -> &'a str {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(FunctionCall::VT_FUNCTION_NAME, None)
                .unwrap()
        }
    }
    #[inline]
    pub fn key_compare_less_than(&self, o: &FunctionCall) -> bool {
        self.function_name() < o.function_name()
    }

    #[inline]
    pub fn key_compare_with_value(&self, val: &str) -> ::core::cmp::Ordering {
        let key = self.function_name();
        key.cmp(val)
    }
    #[inline]
    pub fn parameters(
        &self,
    ) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Parameter<'a>>>> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab.get::<flatbuffers::ForwardsUOffset<
                flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Parameter>>,
            >>(FunctionCall::VT_PARAMETERS, None)
        }
    }
    #[inline]
    pub fn function_call_type(&self) -> FunctionCallType {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<FunctionCallType>(
                    FunctionCall::VT_FUNCTION_CALL_TYPE,
                    Some(FunctionCallType::none),
                )
                .unwrap()
        }
    }
}

impl flatbuffers::Verifiable for FunctionCall<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        use self::flatbuffers::Verifiable;
        v.visit_table(pos)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "function_name",
                Self::VT_FUNCTION_NAME,
                true,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<
                flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<Parameter>>,
            >>("parameters", Self::VT_PARAMETERS, false)?
            .visit_field::<FunctionCallType>(
                "function_call_type",
                Self::VT_FUNCTION_CALL_TYPE,
                false,
            )?
            .finish();
        Ok(())
    }
}
pub struct FunctionCallArgs<'a> {
    pub function_name: Option<flatbuffers::WIPOffset<&'a str>>,
    pub parameters: Option<
        flatbuffers::WIPOffset<
            flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Parameter<'a>>>,
        >,
    >,
    pub function_call_type: FunctionCallType,
}
impl<'a> Default for FunctionCallArgs<'a> {
    #[inline]
    fn default() -> Self {
        FunctionCallArgs {
            function_name: None, // required field
            parameters: None,
            function_call_type: FunctionCallType::none,
        }
    }
}

pub struct FunctionCallBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> FunctionCallBuilder<'a, 'b> {
    #[inline]
    pub fn add_function_name(&mut self, function_name: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            FunctionCall::VT_FUNCTION_NAME,
            function_name,
        );
    }
    #[inline]
    pub fn add_parameters(
        &mut self,
        parameters: flatbuffers::WIPOffset<
            flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<Parameter<'b>>>,
        >,
    ) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(FunctionCall::VT_PARAMETERS, parameters);
    }
    #[inline]
    pub fn add_function_call_type(&mut self, function_call_type: FunctionCallType) {
        self.fbb_.push_slot::<FunctionCallType>(
            FunctionCall::VT_FUNCTION_CALL_TYPE,
            function_call_type,
            FunctionCallType::none,
        );
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> FunctionCallBuilder<'a, 'b> {
        let start = _fbb.start_table();
        FunctionCallBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<FunctionCall<'a>> {
        let o = self.fbb_.end_table(self.start_);
        self.fbb_
            .required(o, FunctionCall::VT_FUNCTION_NAME, "function_name");
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl core::fmt::Debug for FunctionCall<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("FunctionCall");
        ds.field("function_name", &self.function_name());
        ds.field("parameters", &self.parameters());
        ds.field("function_call_type", &self.function_call_type());
        ds.finish()
    }
}
#[inline]
/// Verifies that a buffer of bytes contains a `FunctionCall`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_function_call_unchecked`.
pub fn root_as_function_call(buf: &[u8]) -> Result<FunctionCall, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::root::<FunctionCall>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `FunctionCall` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_function_call_unchecked`.
pub fn size_prefixed_root_as_function_call(
    buf: &[u8],
) -> Result<FunctionCall, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::size_prefixed_root::<FunctionCall>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `FunctionCall` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_function_call_unchecked`.
pub fn root_as_function_call_with_opts<'b, 'o>(
    opts: &'o flatbuffers::VerifierOptions,
    buf: &'b [u8],
) -> Result<FunctionCall<'b>, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::root_with_opts::<FunctionCall<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `FunctionCall` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_function_call_unchecked`.
pub fn size_prefixed_root_as_function_call_with_opts<'b, 'o>(
    opts: &'o flatbuffers::VerifierOptions,
    buf: &'b [u8],
) -> Result<FunctionCall<'b>, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::size_prefixed_root_with_opts::<FunctionCall<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a FunctionCall and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `FunctionCall`.
pub unsafe fn root_as_function_call_unchecked(buf: &[u8]) -> FunctionCall {
    flatbuffers::root_unchecked::<FunctionCall>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed FunctionCall and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `FunctionCall`.
pub unsafe fn size_prefixed_root_as_function_call_unchecked(buf: &[u8]) -> FunctionCall {
    flatbuffers::size_prefixed_root_unchecked::<FunctionCall>(buf)
}
#[inline]
pub fn finish_function_call_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<FunctionCall<'a>>,
) {
    fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_function_call_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<FunctionCall<'a>>,
) {
    fbb.finish_size_prefixed(root, None);
}
