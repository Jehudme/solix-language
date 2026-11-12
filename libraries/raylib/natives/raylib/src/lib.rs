//! Auto-generated 1-to-1 Rust native bridge extension for Solix Raylib bindings.
//! Generated from official raylib_api.json specification.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_variables, dead_code, unused_imports, unused_mut, clippy::all)]

use std::ffi::{c_char, c_int, c_longlong, c_short, c_uint, c_ulonglong, c_ushort, c_void, CStr};

// ===========================================================================
// Upstream Raylib Callback Function Signatures
// ===========================================================================
pub type TraceLogCallback = Option<unsafe extern "C" fn(log_level: c_int, text: *const c_char, args: *mut c_void)>;
pub type LoadFileDataCallback = Option<unsafe extern "C" fn(file_name: *const c_char, data_size: *mut c_int) -> *mut u8>;
pub type SaveFileDataCallback = Option<unsafe extern "C" fn(file_name: *const c_char, data: *const c_void, data_size: c_int) -> bool>;
pub type LoadFileTextCallback = Option<unsafe extern "C" fn(file_name: *const c_char) -> *mut c_char>;
pub type SaveFileTextCallback = Option<unsafe extern "C" fn(file_name: *const c_char, text: *const c_char) -> bool>;
pub type AudioCallback = Option<unsafe extern "C" fn(buffer_data: *mut c_void, frames: c_uint)>;

/// Stable C-ABI tagged value passed across native shared library boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

impl SolixCValue {
    #[inline]
    pub const fn null() -> Self {
        Self { tag: 0, flags: 0, payload: 0 }
    }
    #[inline]
    pub const fn bool_val(val: bool) -> Self {
        Self { tag: 1, flags: 0, payload: if val { 1 } else { 0 } }
    }
    #[inline]
    pub const fn i32_val(val: i32) -> Self {
        Self { tag: 2, flags: 0, payload: (val as i64) as u64 }
    }
    #[inline]
    pub const fn i64_val(val: i64) -> Self {
        Self { tag: 3, flags: 0, payload: val as u64 }
    }
    #[inline]
    pub const fn f64_val(val: f64) -> Self {
        Self { tag: 4, flags: 0, payload: val.to_bits() }
    }
    #[inline]
    pub const fn ref_val(addr: u32) -> Self {
        Self { tag: 5, flags: 0, payload: addr as u64 }
    }
}

/// Context structure passed to native C callbacks providing safe field and heap accessor function pointers.
#[repr(C)]
pub struct SolixVmContext {
    pub magic: u64,
    pub vm_ptr: *mut c_void,
    pub get_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32) -> i32,
    pub set_field_i32: unsafe extern "C" fn(*mut c_void, u32, u32, i32),
    pub read_bytes: unsafe extern "C" fn(vm_ptr: *mut c_void, addr: u32, out_buf: *mut u8, cap: u32) -> u32,
    pub alloc_string: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,
    pub alloc_chars: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,
    pub alloc_byte_array: unsafe extern "C" fn(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32,
    pub alloc_array: unsafe extern "C" fn(vm_ptr: *mut c_void, len: u32, type_tag: u8) -> u32,
    pub array_len: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32) -> u32,
    pub array_get_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32) -> u64,
    pub array_set_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, arr_addr: u32, index: u32, raw_val: u64),
    pub alloc_object: unsafe extern "C" fn(vm_ptr: *mut c_void, class_vtable_id: u32, field_count: u32) -> u32,
    pub read_field_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, offset: u32) -> u64,
    pub write_field_u64: unsafe extern "C" fn(vm_ptr: *mut c_void, obj_addr: u32, offset: u32, raw_val: u64),
}

pub type NativeCFunctionPtr = unsafe extern "C" fn(
    *mut c_void,
    *const SolixCValue,
    u32,
    *mut SolixCValue,
) -> SolixCValue;

pub type RegisterNativeCallback = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    NativeCFunctionPtr,
);

// ===========================================================================
// Raylib C ABI Struct Definitions
// ===========================================================================
pub type C_Texture2D = C_Texture;
pub type C_TextureCubemap = C_Texture;
pub type C_RenderTexture2D = C_RenderTexture;
pub type C_Camera = C_Camera3D;
pub type C_Quaternion = C_Vector4;
pub type C_ModelAnimPose = *mut C_Transform;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for C_Vector2 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for C_Vector3 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for C_Vector4 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Matrix {
    pub m0: f32,
    pub m4: f32,
    pub m8: f32,
    pub m12: f32,
    pub m1: f32,
    pub m5: f32,
    pub m9: f32,
    pub m13: f32,
    pub m2: f32,
    pub m6: f32,
    pub m10: f32,
    pub m14: f32,
    pub m3: f32,
    pub m7: f32,
    pub m11: f32,
    pub m15: f32,
}

impl Default for C_Matrix {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for C_Color {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for C_Rectangle {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Image {
    pub data: *mut c_void,
    pub width: c_int,
    pub height: c_int,
    pub mipmaps: c_int,
    pub format: c_int,
}

impl Default for C_Image {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Texture {
    pub id: c_uint,
    pub width: c_int,
    pub height: c_int,
    pub mipmaps: c_int,
    pub format: c_int,
}

impl Default for C_Texture {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_RenderTexture {
    pub id: c_uint,
    pub texture: C_Texture,
    pub depth: C_Texture,
}

impl Default for C_RenderTexture {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_NPatchInfo {
    pub source: C_Rectangle,
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
    pub layout: c_int,
}

impl Default for C_NPatchInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_GlyphInfo {
    pub value: c_int,
    pub offsetX: c_int,
    pub offsetY: c_int,
    pub advanceX: c_int,
    pub image: C_Image,
}

impl Default for C_GlyphInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Font {
    pub baseSize: c_int,
    pub glyphCount: c_int,
    pub glyphPadding: c_int,
    pub texture: C_Texture2D,
    pub recs: *mut C_Rectangle,
    pub glyphs: *mut C_GlyphInfo,
}

impl Default for C_Font {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Camera3D {
    pub position: C_Vector3,
    pub target: C_Vector3,
    pub up: C_Vector3,
    pub fovy: f32,
    pub projection: c_int,
}

impl Default for C_Camera3D {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Camera2D {
    pub offset: C_Vector2,
    pub target: C_Vector2,
    pub rotation: f32,
    pub zoom: f32,
}

impl Default for C_Camera2D {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Mesh {
    pub vertexCount: c_int,
    pub triangleCount: c_int,
    pub vertices: *mut f32,
    pub texcoords: *mut f32,
    pub texcoords2: *mut f32,
    pub normals: *mut f32,
    pub tangents: *mut f32,
    pub colors: *mut u8,
    pub indices: *mut c_ushort,
    pub boneCount: c_int,
    pub boneIndices: *mut u8,
    pub boneWeights: *mut f32,
    pub animVertices: *mut f32,
    pub animNormals: *mut f32,
    pub vaoId: c_uint,
    pub vboId: *mut c_uint,
}

impl Default for C_Mesh {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Shader {
    pub id: c_uint,
    pub locs: *mut c_int,
}

impl Default for C_Shader {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_MaterialMap {
    pub texture: C_Texture2D,
    pub color: C_Color,
    pub value: f32,
}

impl Default for C_MaterialMap {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Material {
    pub shader: C_Shader,
    pub maps: *mut C_MaterialMap,
    pub params: [f32; 4],
}

impl Default for C_Material {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Transform {
    pub translation: C_Vector3,
    pub rotation: C_Quaternion,
    pub scale: C_Vector3,
}

impl Default for C_Transform {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_BoneInfo {
    pub name: [c_char; 32],
    pub parent: c_int,
}

impl Default for C_BoneInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_ModelSkeleton {
    pub boneCount: c_uint,
    pub bones: *mut C_BoneInfo,
    pub bindPose: *mut C_Transform,
}

impl Default for C_ModelSkeleton {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Model {
    pub transform: C_Matrix,
    pub meshCount: c_int,
    pub materialCount: c_int,
    pub meshes: *mut C_Mesh,
    pub materials: *mut C_Material,
    pub meshMaterial: *mut c_int,
    pub skeleton: C_ModelSkeleton,
    pub currentPose: *mut C_Transform,
    pub boneMatrices: *mut C_Matrix,
}

impl Default for C_Model {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_ModelAnimation {
    pub name: [c_char; 32],
    pub boneCount: c_uint,
    pub keyframeCount: c_int,
    pub keyframePoses: *mut *mut C_Transform,
}

impl Default for C_ModelAnimation {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Ray {
    pub position: C_Vector3,
    pub direction: C_Vector3,
}

impl Default for C_Ray {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_RayCollision {
    pub hit: bool,
    pub distance: f32,
    pub point: C_Vector3,
    pub normal: C_Vector3,
}

impl Default for C_RayCollision {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_BoundingBox {
    pub min: C_Vector3,
    pub max: C_Vector3,
}

impl Default for C_BoundingBox {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Wave {
    pub frameCount: c_uint,
    pub sampleRate: c_uint,
    pub sampleSize: c_uint,
    pub channels: c_uint,
    pub data: *mut c_void,
}

impl Default for C_Wave {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_AudioStream {
    pub buffer: *mut c_void,
    pub processor: *mut c_void,
    pub sampleRate: c_uint,
    pub sampleSize: c_uint,
    pub channels: c_uint,
}

impl Default for C_AudioStream {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Sound {
    pub stream: C_AudioStream,
    pub frameCount: c_uint,
}

impl Default for C_Sound {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_Music {
    pub stream: C_AudioStream,
    pub frameCount: c_uint,
    pub looping: bool,
    pub ctxType: c_int,
    pub ctxData: *mut c_void,
}

impl Default for C_Music {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_VrDeviceInfo {
    pub hResolution: c_int,
    pub vResolution: c_int,
    pub hScreenSize: f32,
    pub vScreenSize: f32,
    pub eyeToScreenDistance: f32,
    pub lensSeparationDistance: f32,
    pub interpupillaryDistance: f32,
    pub lensDistortionValues: [f32; 4],
    pub chromaAbCorrection: [f32; 4],
}

impl Default for C_VrDeviceInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_VrStereoConfig {
    pub projection: [C_Matrix; 2],
    pub viewOffset: [C_Matrix; 2],
    pub leftLensCenter: [f32; 2],
    pub rightLensCenter: [f32; 2],
    pub leftScreenCenter: [f32; 2],
    pub rightScreenCenter: [f32; 2],
    pub scale: [f32; 2],
    pub scaleIn: [f32; 2],
}

impl Default for C_VrStereoConfig {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_FilePathList {
    pub count: c_uint,
    pub paths: *mut *mut c_char,
}

impl Default for C_FilePathList {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_AutomationEvent {
    pub frame: c_uint,
    pub r#type: c_uint,
    pub params: [c_int; 4],
}

impl Default for C_AutomationEvent {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_AutomationEventList {
    pub capacity: c_uint,
    pub count: c_uint,
    pub events: *mut C_AutomationEvent,
}

impl Default for C_AutomationEventList {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// ===========================================================================
// Dynamic Struct Unpack, Pack & Writeback Helpers
// ===========================================================================

unsafe fn writeback_vector2(vm_context: *mut c_void, addr: u32, val: &C_Vector2) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.x.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.y.to_bits() as u64);
}

unsafe fn pack_vector2(vm_context: *mut c_void, val: C_Vector2) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_vector2(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vector2(vm_context: *mut c_void, addr: u32) -> C_Vector2 {
    if addr == 0 { return C_Vector2::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Vector2::default();
    result.x = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.y = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result
}

unsafe fn writeback_vector3(vm_context: *mut c_void, addr: u32, val: &C_Vector3) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.x.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.y.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.z.to_bits() as u64);
}

unsafe fn pack_vector3(vm_context: *mut c_void, val: C_Vector3) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_vector3(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vector3(vm_context: *mut c_void, addr: u32) -> C_Vector3 {
    if addr == 0 { return C_Vector3::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Vector3::default();
    result.x = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.y = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.z = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result
}

unsafe fn writeback_vector4(vm_context: *mut c_void, addr: u32, val: &C_Vector4) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.x.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.y.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.z.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.w.to_bits() as u64);
}

unsafe fn pack_vector4(vm_context: *mut c_void, val: C_Vector4) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 4);
    if addr > 0 {
        writeback_vector4(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vector4(vm_context: *mut c_void, addr: u32) -> C_Vector4 {
    if addr == 0 { return C_Vector4::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Vector4::default();
    result.x = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.y = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.z = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.w = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result
}

unsafe fn writeback_matrix(vm_context: *mut c_void, addr: u32, val: &C_Matrix) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.m0.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.m4.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.m8.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.m12.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.m1.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 5, val.m5.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 6, val.m9.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 7, val.m13.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 8, val.m2.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 9, val.m6.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 10, val.m10.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 11, val.m14.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 12, val.m3.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 13, val.m7.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 14, val.m11.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 15, val.m15.to_bits() as u64);
}

unsafe fn pack_matrix(vm_context: *mut c_void, val: C_Matrix) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 16);
    if addr > 0 {
        writeback_matrix(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_matrix(vm_context: *mut c_void, addr: u32) -> C_Matrix {
    if addr == 0 { return C_Matrix::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Matrix::default();
    result.m0 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.m4 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.m8 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.m12 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result.m1 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 4) as u32);
    result.m5 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 5) as u32);
    result.m9 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 6) as u32);
    result.m13 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 7) as u32);
    result.m2 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 8) as u32);
    result.m6 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 9) as u32);
    result.m10 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 10) as u32);
    result.m14 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 11) as u32);
    result.m3 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 12) as u32);
    result.m7 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 13) as u32);
    result.m11 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 14) as u32);
    result.m15 = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 15) as u32);
    result
}

unsafe fn writeback_color(vm_context: *mut c_void, addr: u32, val: &C_Color) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.r as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.g as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.b as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.a as i32);
}

unsafe fn pack_color(vm_context: *mut c_void, val: C_Color) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 4);
    if addr > 0 {
        writeback_color(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_color(vm_context: *mut c_void, addr: u32) -> C_Color {
    if addr == 0 { return C_Color::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Color::default();
    result.r = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as u8;
    result.g = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as u8;
    result.b = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as u8;
    result.a = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as u8;
    result
}

unsafe fn writeback_rectangle(vm_context: *mut c_void, addr: u32, val: &C_Rectangle) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.x.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.y.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.width.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.height.to_bits() as u64);
}

unsafe fn pack_rectangle(vm_context: *mut c_void, val: C_Rectangle) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 4);
    if addr > 0 {
        writeback_rectangle(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_rectangle(vm_context: *mut c_void, addr: u32) -> C_Rectangle {
    if addr == 0 { return C_Rectangle::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Rectangle::default();
    result.x = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.y = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.width = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.height = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result
}

unsafe fn writeback_image(vm_context: *mut c_void, addr: u32, val: &C_Image) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.data as usize as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.width as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.height as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.mipmaps as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 4, val.format as i32);
}

unsafe fn pack_image(vm_context: *mut c_void, val: C_Image) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_image(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_image(vm_context: *mut c_void, addr: u32) -> C_Image {
    if addr == 0 { return C_Image::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Image::default();
    result.data = ((*context_ptr).read_field_u64)(vm_context, addr, 0) as *mut c_void;
    result.width = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.height = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.mipmaps = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_int;
    result.format = ((*context_ptr).get_field_i32)(vm_context, addr, 4) as c_int;
    result
}

unsafe fn writeback_texture(vm_context: *mut c_void, addr: u32, val: &C_Texture) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.id as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.width as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.height as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.mipmaps as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 4, val.format as i32);
}

unsafe fn pack_texture(vm_context: *mut c_void, val: C_Texture) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_texture(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_texture(vm_context: *mut c_void, addr: u32) -> C_Texture {
    if addr == 0 { return C_Texture::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Texture::default();
    result.id = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.width = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.height = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.mipmaps = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_int;
    result.format = ((*context_ptr).get_field_i32)(vm_context, addr, 4) as c_int;
    result
}

unsafe fn writeback_render_texture(vm_context: *mut c_void, addr: u32, val: &C_RenderTexture) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.id as i32);
    let child_1 = pack_texture(vm_context, val.texture);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
    let child_2 = pack_texture(vm_context, val.depth);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, child_2 as u64);
}

unsafe fn pack_render_texture(vm_context: *mut c_void, val: C_RenderTexture) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_render_texture(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_render_texture(vm_context: *mut c_void, addr: u32) -> C_RenderTexture {
    if addr == 0 { return C_RenderTexture::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_RenderTexture::default();
    result.id = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.texture = unpack_texture(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.depth = unpack_texture(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result
}

unsafe fn writeback_n_patch_info(vm_context: *mut c_void, addr: u32, val: &C_NPatchInfo) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_rectangle(vm_context, val.source);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.left as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.top as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.right as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 4, val.bottom as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 5, val.layout as i32);
}

unsafe fn pack_n_patch_info(vm_context: *mut c_void, val: C_NPatchInfo) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 6);
    if addr > 0 {
        writeback_n_patch_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_n_patch_info(vm_context: *mut c_void, addr: u32) -> C_NPatchInfo {
    if addr == 0 { return C_NPatchInfo::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_NPatchInfo::default();
    result.source = unpack_rectangle(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.left = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.top = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.right = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_int;
    result.bottom = ((*context_ptr).get_field_i32)(vm_context, addr, 4) as c_int;
    result.layout = ((*context_ptr).get_field_i32)(vm_context, addr, 5) as c_int;
    result
}

unsafe fn writeback_glyph_info(vm_context: *mut c_void, addr: u32, val: &C_GlyphInfo) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.value as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.offsetX as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.offsetY as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.advanceX as i32);
    let child_4 = pack_image(vm_context, val.image);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, child_4 as u64);
}

unsafe fn pack_glyph_info(vm_context: *mut c_void, val: C_GlyphInfo) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_glyph_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_glyph_info(vm_context: *mut c_void, addr: u32) -> C_GlyphInfo {
    if addr == 0 { return C_GlyphInfo::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_GlyphInfo::default();
    result.value = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_int;
    result.offsetX = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.offsetY = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.advanceX = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_int;
    result.image = unpack_image(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 4) as u32);
    result
}

unsafe fn writeback_font(vm_context: *mut c_void, addr: u32, val: &C_Font) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.baseSize as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.glyphCount as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.glyphPadding as i32);
    let child_3 = pack_texture(vm_context, val.texture);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, child_3 as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.recs as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 5, val.glyphs as usize as u64);
}

unsafe fn pack_font(vm_context: *mut c_void, val: C_Font) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 6);
    if addr > 0 {
        writeback_font(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_font(vm_context: *mut c_void, addr: u32) -> C_Font {
    if addr == 0 { return C_Font::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Font::default();
    result.baseSize = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_int;
    result.glyphCount = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.glyphPadding = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.texture = unpack_texture(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result.recs = ((*context_ptr).read_field_u64)(vm_context, addr, 4) as *mut C_Rectangle;
    result.glyphs = ((*context_ptr).read_field_u64)(vm_context, addr, 5) as *mut C_GlyphInfo;
    result
}

unsafe fn writeback_camera3d(vm_context: *mut c_void, addr: u32, val: &C_Camera3D) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_vector3(vm_context, val.position);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    let child_1 = pack_vector3(vm_context, val.target);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
    let child_2 = pack_vector3(vm_context, val.up);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, child_2 as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.fovy.to_bits() as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 4, val.projection as i32);
}

unsafe fn pack_camera3d(vm_context: *mut c_void, val: C_Camera3D) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_camera3d(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_camera3d(vm_context: *mut c_void, addr: u32) -> C_Camera3D {
    if addr == 0 { return C_Camera3D::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Camera3D::default();
    result.position = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.target = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.up = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.fovy = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result.projection = ((*context_ptr).get_field_i32)(vm_context, addr, 4) as c_int;
    result
}

unsafe fn writeback_camera2d(vm_context: *mut c_void, addr: u32, val: &C_Camera2D) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_vector2(vm_context, val.offset);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    let child_1 = pack_vector2(vm_context, val.target);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.rotation.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.zoom.to_bits() as u64);
}

unsafe fn pack_camera2d(vm_context: *mut c_void, val: C_Camera2D) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 4);
    if addr > 0 {
        writeback_camera2d(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_camera2d(vm_context: *mut c_void, addr: u32) -> C_Camera2D {
    if addr == 0 { return C_Camera2D::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Camera2D::default();
    result.offset = unpack_vector2(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.target = unpack_vector2(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.rotation = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.zoom = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result
}

unsafe fn writeback_mesh(vm_context: *mut c_void, addr: u32, val: &C_Mesh) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.vertexCount as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.triangleCount as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.vertices as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.texcoords as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.texcoords2 as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 5, val.normals as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 6, val.tangents as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 7, val.colors as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 8, val.indices as usize as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 9, val.boneCount as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 10, val.boneIndices as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 11, val.boneWeights as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 12, val.animVertices as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 13, val.animNormals as usize as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 14, val.vaoId as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 15, val.vboId as usize as u64);
}

unsafe fn pack_mesh(vm_context: *mut c_void, val: C_Mesh) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 16);
    if addr > 0 {
        writeback_mesh(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_mesh(vm_context: *mut c_void, addr: u32) -> C_Mesh {
    if addr == 0 { return C_Mesh::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Mesh::default();
    result.vertexCount = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_int;
    result.triangleCount = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.vertices = ((*context_ptr).read_field_u64)(vm_context, addr, 2) as *mut f32;
    result.texcoords = ((*context_ptr).read_field_u64)(vm_context, addr, 3) as *mut f32;
    result.texcoords2 = ((*context_ptr).read_field_u64)(vm_context, addr, 4) as *mut f32;
    result.normals = ((*context_ptr).read_field_u64)(vm_context, addr, 5) as *mut f32;
    result.tangents = ((*context_ptr).read_field_u64)(vm_context, addr, 6) as *mut f32;
    result.colors = ((*context_ptr).read_field_u64)(vm_context, addr, 7) as *mut u8;
    result.indices = ((*context_ptr).read_field_u64)(vm_context, addr, 8) as *mut c_ushort;
    result.boneCount = ((*context_ptr).get_field_i32)(vm_context, addr, 9) as c_int;
    result.boneIndices = ((*context_ptr).read_field_u64)(vm_context, addr, 10) as *mut u8;
    result.boneWeights = ((*context_ptr).read_field_u64)(vm_context, addr, 11) as *mut f32;
    result.animVertices = ((*context_ptr).read_field_u64)(vm_context, addr, 12) as *mut f32;
    result.animNormals = ((*context_ptr).read_field_u64)(vm_context, addr, 13) as *mut f32;
    result.vaoId = ((*context_ptr).get_field_i32)(vm_context, addr, 14) as c_uint;
    result.vboId = ((*context_ptr).read_field_u64)(vm_context, addr, 15) as *mut c_uint;
    result
}

unsafe fn writeback_shader(vm_context: *mut c_void, addr: u32, val: &C_Shader) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.id as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.locs as usize as u64);
}

unsafe fn pack_shader(vm_context: *mut c_void, val: C_Shader) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_shader(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_shader(vm_context: *mut c_void, addr: u32) -> C_Shader {
    if addr == 0 { return C_Shader::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Shader::default();
    result.id = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.locs = ((*context_ptr).read_field_u64)(vm_context, addr, 1) as *mut c_int;
    result
}

unsafe fn writeback_material_map(vm_context: *mut c_void, addr: u32, val: &C_MaterialMap) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_texture(vm_context, val.texture);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    let child_1 = pack_color(vm_context, val.color);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.value.to_bits() as u64);
}

unsafe fn pack_material_map(vm_context: *mut c_void, val: C_MaterialMap) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_material_map(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_material_map(vm_context: *mut c_void, addr: u32) -> C_MaterialMap {
    if addr == 0 { return C_MaterialMap::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_MaterialMap::default();
    result.texture = unpack_texture(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.color = unpack_color(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.value = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result
}

unsafe fn writeback_material(vm_context: *mut c_void, addr: u32, val: &C_Material) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_shader(vm_context, val.shader);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.maps as usize as u64);
}

unsafe fn pack_material(vm_context: *mut c_void, val: C_Material) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_material(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_material(vm_context: *mut c_void, addr: u32) -> C_Material {
    if addr == 0 { return C_Material::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Material::default();
    result.shader = unpack_shader(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.maps = ((*context_ptr).read_field_u64)(vm_context, addr, 1) as *mut C_MaterialMap;
    result
}

unsafe fn writeback_transform(vm_context: *mut c_void, addr: u32, val: &C_Transform) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_vector3(vm_context, val.translation);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    let child_1 = pack_vector4(vm_context, val.rotation);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
    let child_2 = pack_vector3(vm_context, val.scale);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, child_2 as u64);
}

unsafe fn pack_transform(vm_context: *mut c_void, val: C_Transform) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_transform(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_transform(vm_context: *mut c_void, addr: u32) -> C_Transform {
    if addr == 0 { return C_Transform::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Transform::default();
    result.translation = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.rotation = unpack_vector4(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.scale = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result
}

unsafe fn writeback_bone_info(vm_context: *mut c_void, addr: u32, val: &C_BoneInfo) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.parent as i32);
}

unsafe fn pack_bone_info(vm_context: *mut c_void, val: C_BoneInfo) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_bone_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_bone_info(vm_context: *mut c_void, addr: u32) -> C_BoneInfo {
    if addr == 0 { return C_BoneInfo::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_BoneInfo::default();
    result.parent = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result
}

unsafe fn writeback_model_skeleton(vm_context: *mut c_void, addr: u32, val: &C_ModelSkeleton) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.boneCount as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.bones as usize as u64);
}

unsafe fn pack_model_skeleton(vm_context: *mut c_void, val: C_ModelSkeleton) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_model_skeleton(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_model_skeleton(vm_context: *mut c_void, addr: u32) -> C_ModelSkeleton {
    if addr == 0 { return C_ModelSkeleton::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_ModelSkeleton::default();
    result.boneCount = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.bones = ((*context_ptr).read_field_u64)(vm_context, addr, 1) as *mut C_BoneInfo;
    result
}

unsafe fn writeback_model(vm_context: *mut c_void, addr: u32, val: &C_Model) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_matrix(vm_context, val.transform);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.meshCount as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.materialCount as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.meshes as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.materials as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 5, val.meshMaterial as usize as u64);
    let child_6 = pack_model_skeleton(vm_context, val.skeleton);
    ((*context_ptr).write_field_u64)(vm_context, addr, 6, child_6 as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 8, val.boneMatrices as usize as u64);
}

unsafe fn pack_model(vm_context: *mut c_void, val: C_Model) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 9);
    if addr > 0 {
        writeback_model(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_model(vm_context: *mut c_void, addr: u32) -> C_Model {
    if addr == 0 { return C_Model::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Model::default();
    result.transform = unpack_matrix(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.meshCount = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.materialCount = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.meshes = ((*context_ptr).read_field_u64)(vm_context, addr, 3) as *mut C_Mesh;
    result.materials = ((*context_ptr).read_field_u64)(vm_context, addr, 4) as *mut C_Material;
    result.meshMaterial = ((*context_ptr).read_field_u64)(vm_context, addr, 5) as *mut c_int;
    result.skeleton = unpack_model_skeleton(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 6) as u32);
    result.boneMatrices = ((*context_ptr).read_field_u64)(vm_context, addr, 8) as *mut C_Matrix;
    result
}

unsafe fn writeback_model_animation(vm_context: *mut c_void, addr: u32, val: &C_ModelAnimation) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.boneCount as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.keyframeCount as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.keyframePoses as usize as u64);
}

unsafe fn pack_model_animation(vm_context: *mut c_void, val: C_ModelAnimation) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 4);
    if addr > 0 {
        writeback_model_animation(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_model_animation(vm_context: *mut c_void, addr: u32) -> C_ModelAnimation {
    if addr == 0 { return C_ModelAnimation::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_ModelAnimation::default();
    result.boneCount = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_uint;
    result.keyframeCount = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_int;
    result.keyframePoses = ((*context_ptr).read_field_u64)(vm_context, addr, 3) as *mut *mut C_Transform;
    result
}

unsafe fn writeback_ray(vm_context: *mut c_void, addr: u32, val: &C_Ray) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_vector3(vm_context, val.position);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    let child_1 = pack_vector3(vm_context, val.direction);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
}

unsafe fn pack_ray(vm_context: *mut c_void, val: C_Ray) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_ray(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_ray(vm_context: *mut c_void, addr: u32) -> C_Ray {
    if addr == 0 { return C_Ray::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Ray::default();
    result.position = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.direction = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result
}

unsafe fn writeback_ray_collision(vm_context: *mut c_void, addr: u32, val: &C_RayCollision) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, if val.hit { 1 } else { 0 });
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.distance.to_bits() as u64);
    let child_2 = pack_vector3(vm_context, val.point);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, child_2 as u64);
    let child_3 = pack_vector3(vm_context, val.normal);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, child_3 as u64);
}

unsafe fn pack_ray_collision(vm_context: *mut c_void, val: C_RayCollision) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 4);
    if addr > 0 {
        writeback_ray_collision(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_ray_collision(vm_context: *mut c_void, addr: u32) -> C_RayCollision {
    if addr == 0 { return C_RayCollision::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_RayCollision::default();
    result.hit = ((*context_ptr).get_field_i32)(vm_context, addr, 0) != 0;
    result.distance = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result.point = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.normal = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result
}

unsafe fn writeback_bounding_box(vm_context: *mut c_void, addr: u32, val: &C_BoundingBox) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_vector3(vm_context, val.min);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    let child_1 = pack_vector3(vm_context, val.max);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, child_1 as u64);
}

unsafe fn pack_bounding_box(vm_context: *mut c_void, val: C_BoundingBox) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_bounding_box(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_bounding_box(vm_context: *mut c_void, addr: u32) -> C_BoundingBox {
    if addr == 0 { return C_BoundingBox::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_BoundingBox::default();
    result.min = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.max = unpack_vector3(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 1) as u32);
    result
}

unsafe fn writeback_wave(vm_context: *mut c_void, addr: u32, val: &C_Wave) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.frameCount as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.sampleRate as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.sampleSize as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.channels as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.data as usize as u64);
}

unsafe fn pack_wave(vm_context: *mut c_void, val: C_Wave) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_wave(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_wave(vm_context: *mut c_void, addr: u32) -> C_Wave {
    if addr == 0 { return C_Wave::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Wave::default();
    result.frameCount = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.sampleRate = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_uint;
    result.sampleSize = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_uint;
    result.channels = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_uint;
    result.data = ((*context_ptr).read_field_u64)(vm_context, addr, 4) as *mut c_void;
    result
}

unsafe fn writeback_audio_stream(vm_context: *mut c_void, addr: u32, val: &C_AudioStream) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, val.buffer as usize as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.processor as usize as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, val.sampleRate as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.sampleSize as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 4, val.channels as i32);
}

unsafe fn pack_audio_stream(vm_context: *mut c_void, val: C_AudioStream) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_audio_stream(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_audio_stream(vm_context: *mut c_void, addr: u32) -> C_AudioStream {
    if addr == 0 { return C_AudioStream::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_AudioStream::default();
    result.buffer = ((*context_ptr).read_field_u64)(vm_context, addr, 0) as *mut c_void;
    result.processor = ((*context_ptr).read_field_u64)(vm_context, addr, 1) as *mut c_void;
    result.sampleRate = ((*context_ptr).get_field_i32)(vm_context, addr, 2) as c_uint;
    result.sampleSize = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_uint;
    result.channels = ((*context_ptr).get_field_i32)(vm_context, addr, 4) as c_uint;
    result
}

unsafe fn writeback_sound(vm_context: *mut c_void, addr: u32, val: &C_Sound) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_audio_stream(vm_context, val.stream);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.frameCount as i32);
}

unsafe fn pack_sound(vm_context: *mut c_void, val: C_Sound) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_sound(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_sound(vm_context: *mut c_void, addr: u32) -> C_Sound {
    if addr == 0 { return C_Sound::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Sound::default();
    result.stream = unpack_audio_stream(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.frameCount = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_uint;
    result
}

unsafe fn writeback_music(vm_context: *mut c_void, addr: u32, val: &C_Music) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    let child_0 = pack_audio_stream(vm_context, val.stream);
    ((*context_ptr).write_field_u64)(vm_context, addr, 0, child_0 as u64);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.frameCount as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 2, if val.looping { 1 } else { 0 });
    ((*context_ptr).set_field_i32)(vm_context, addr, 3, val.ctxType as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.ctxData as usize as u64);
}

unsafe fn pack_music(vm_context: *mut c_void, val: C_Music) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 5);
    if addr > 0 {
        writeback_music(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_music(vm_context: *mut c_void, addr: u32) -> C_Music {
    if addr == 0 { return C_Music::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_Music::default();
    result.stream = unpack_audio_stream(vm_context, ((*context_ptr).read_field_u64)(vm_context, addr, 0) as u32);
    result.frameCount = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_uint;
    result.looping = ((*context_ptr).get_field_i32)(vm_context, addr, 2) != 0;
    result.ctxType = ((*context_ptr).get_field_i32)(vm_context, addr, 3) as c_int;
    result.ctxData = ((*context_ptr).read_field_u64)(vm_context, addr, 4) as *mut c_void;
    result
}

unsafe fn writeback_vr_device_info(vm_context: *mut c_void, addr: u32, val: &C_VrDeviceInfo) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.hResolution as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.vResolution as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.hScreenSize.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 3, val.vScreenSize.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 4, val.eyeToScreenDistance.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 5, val.lensSeparationDistance.to_bits() as u64);
    ((*context_ptr).write_field_u64)(vm_context, addr, 6, val.interpupillaryDistance.to_bits() as u64);
}

unsafe fn pack_vr_device_info(vm_context: *mut c_void, val: C_VrDeviceInfo) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 9);
    if addr > 0 {
        writeback_vr_device_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vr_device_info(vm_context: *mut c_void, addr: u32) -> C_VrDeviceInfo {
    if addr == 0 { return C_VrDeviceInfo::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_VrDeviceInfo::default();
    result.hResolution = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_int;
    result.vResolution = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_int;
    result.hScreenSize = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 2) as u32);
    result.vScreenSize = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 3) as u32);
    result.eyeToScreenDistance = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 4) as u32);
    result.lensSeparationDistance = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 5) as u32);
    result.interpupillaryDistance = f32::from_bits(((*context_ptr).read_field_u64)(vm_context, addr, 6) as u32);
    result
}

unsafe fn writeback_vr_stereo_config(vm_context: *mut c_void, addr: u32, val: &C_VrStereoConfig) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
}

unsafe fn pack_vr_stereo_config(vm_context: *mut c_void, val: C_VrStereoConfig) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 8);
    if addr > 0 {
        writeback_vr_stereo_config(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vr_stereo_config(vm_context: *mut c_void, addr: u32) -> C_VrStereoConfig {
    if addr == 0 { return C_VrStereoConfig::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_VrStereoConfig::default();
    result
}

unsafe fn writeback_file_path_list(vm_context: *mut c_void, addr: u32, val: &C_FilePathList) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.count as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 1, val.paths as usize as u64);
}

unsafe fn pack_file_path_list(vm_context: *mut c_void, val: C_FilePathList) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 2);
    if addr > 0 {
        writeback_file_path_list(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_file_path_list(vm_context: *mut c_void, addr: u32) -> C_FilePathList {
    if addr == 0 { return C_FilePathList::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_FilePathList::default();
    result.count = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.paths = ((*context_ptr).read_field_u64)(vm_context, addr, 1) as *mut *mut c_char;
    result
}

unsafe fn writeback_automation_event(vm_context: *mut c_void, addr: u32, val: &C_AutomationEvent) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.frame as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.r#type as i32);
}

unsafe fn pack_automation_event(vm_context: *mut c_void, val: C_AutomationEvent) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_automation_event(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_automation_event(vm_context: *mut c_void, addr: u32) -> C_AutomationEvent {
    if addr == 0 { return C_AutomationEvent::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_AutomationEvent::default();
    result.frame = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.r#type = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_uint;
    result
}

unsafe fn writeback_automation_event_list(vm_context: *mut c_void, addr: u32, val: &C_AutomationEventList) {
    if addr == 0 { return; }
    let context_ptr = vm_context as *mut SolixVmContext;
    ((*context_ptr).set_field_i32)(vm_context, addr, 0, val.capacity as i32);
    ((*context_ptr).set_field_i32)(vm_context, addr, 1, val.count as i32);
    ((*context_ptr).write_field_u64)(vm_context, addr, 2, val.events as usize as u64);
}

unsafe fn pack_automation_event_list(vm_context: *mut c_void, val: C_AutomationEventList) -> u32 {
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_object)(vm_context, 0, 3);
    if addr > 0 {
        writeback_automation_event_list(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_automation_event_list(vm_context: *mut c_void, addr: u32) -> C_AutomationEventList {
    if addr == 0 { return C_AutomationEventList::default(); }
    let context_ptr = vm_context as *mut SolixVmContext;
    let mut result = C_AutomationEventList::default();
    result.capacity = ((*context_ptr).get_field_i32)(vm_context, addr, 0) as c_uint;
    result.count = ((*context_ptr).get_field_i32)(vm_context, addr, 1) as c_uint;
    result.events = ((*context_ptr).read_field_u64)(vm_context, addr, 2) as *mut C_AutomationEvent;
    result
}

unsafe fn unpack_quaternion(vm_context: *mut c_void, addr: u32) -> C_Quaternion { unpack_vector4(vm_context, addr) }
unsafe fn pack_quaternion(vm_context: *mut c_void, val: C_Quaternion) -> u32 { pack_vector4(vm_context, val) }
unsafe fn writeback_quaternion(vm_context: *mut c_void, addr: u32, val: &C_Quaternion) { writeback_vector4(vm_context, addr, val); }
unsafe fn unpack_texture2d(vm_context: *mut c_void, addr: u32) -> C_Texture2D { unpack_texture(vm_context, addr) }
unsafe fn pack_texture2d(vm_context: *mut c_void, val: C_Texture2D) -> u32 { pack_texture(vm_context, val) }
unsafe fn writeback_texture2d(vm_context: *mut c_void, addr: u32, val: &C_Texture2D) { writeback_texture(vm_context, addr, val); }
unsafe fn unpack_texture_cubemap(vm_context: *mut c_void, addr: u32) -> C_TextureCubemap { unpack_texture(vm_context, addr) }
unsafe fn pack_texture_cubemap(vm_context: *mut c_void, val: C_TextureCubemap) -> u32 { pack_texture(vm_context, val) }
unsafe fn writeback_texture_cubemap(vm_context: *mut c_void, addr: u32, val: &C_TextureCubemap) { writeback_texture(vm_context, addr, val); }
unsafe fn unpack_render_texture2d(vm_context: *mut c_void, addr: u32) -> C_RenderTexture2D { unpack_render_texture(vm_context, addr) }
unsafe fn pack_render_texture2d(vm_context: *mut c_void, val: C_RenderTexture2D) -> u32 { pack_render_texture(vm_context, val) }
unsafe fn writeback_render_texture2d(vm_context: *mut c_void, addr: u32, val: &C_RenderTexture2D) { writeback_render_texture(vm_context, addr, val); }
unsafe fn unpack_camera(vm_context: *mut c_void, addr: u32) -> C_Camera { unpack_camera3d(vm_context, addr) }
unsafe fn pack_camera(vm_context: *mut c_void, val: C_Camera) -> u32 { pack_camera3d(vm_context, val) }
unsafe fn writeback_camera(vm_context: *mut c_void, addr: u32, val: &C_Camera) { writeback_camera3d(vm_context, addr, val); }

// ===========================================================================
// Upstream Raylib Extern "C" Declarations
// ===========================================================================
unsafe extern "C" {
    pub fn InitWindow(arg_0: c_int, arg_1: c_int, arg_2: *const c_char);
    pub fn CloseWindow();
    pub fn WindowShouldClose() -> bool;
    pub fn IsWindowReady() -> bool;
    pub fn IsWindowFullscreen() -> bool;
    pub fn IsWindowHidden() -> bool;
    pub fn IsWindowMinimized() -> bool;
    pub fn IsWindowMaximized() -> bool;
    pub fn IsWindowFocused() -> bool;
    pub fn IsWindowResized() -> bool;
    pub fn IsWindowState(arg_0: c_uint) -> bool;
    pub fn SetWindowState(arg_0: c_uint);
    pub fn ClearWindowState(arg_0: c_uint);
    pub fn ToggleFullscreen();
    pub fn ToggleBorderlessWindowed();
    pub fn MaximizeWindow();
    pub fn MinimizeWindow();
    pub fn RestoreWindow();
    pub fn SetWindowIcon(arg_0: C_Image);
    pub fn SetWindowIcons(arg_0: *mut C_Image, arg_1: c_int);
    pub fn SetWindowTitle(arg_0: *const c_char);
    pub fn SetWindowPosition(arg_0: c_int, arg_1: c_int);
    pub fn SetWindowMonitor(arg_0: c_int);
    pub fn SetWindowMinSize(arg_0: c_int, arg_1: c_int);
    pub fn SetWindowMaxSize(arg_0: c_int, arg_1: c_int);
    pub fn SetWindowSize(arg_0: c_int, arg_1: c_int);
    pub fn SetWindowOpacity(arg_0: f32);
    pub fn SetWindowFocused();
    pub fn GetWindowHandle() -> *mut c_void;
    pub fn GetScreenWidth() -> c_int;
    pub fn GetScreenHeight() -> c_int;
    pub fn GetRenderWidth() -> c_int;
    pub fn GetRenderHeight() -> c_int;
    pub fn GetMonitorCount() -> c_int;
    pub fn GetCurrentMonitor() -> c_int;
    pub fn GetMonitorPosition(arg_0: c_int) -> C_Vector2;
    pub fn GetMonitorWidth(arg_0: c_int) -> c_int;
    pub fn GetMonitorHeight(arg_0: c_int) -> c_int;
    pub fn GetMonitorPhysicalWidth(arg_0: c_int) -> c_int;
    pub fn GetMonitorPhysicalHeight(arg_0: c_int) -> c_int;
    pub fn GetMonitorRefreshRate(arg_0: c_int) -> c_int;
    pub fn GetWindowPosition() -> C_Vector2;
    pub fn GetWindowScaleDPI() -> C_Vector2;
    pub fn GetMonitorName(arg_0: c_int) -> *const c_char;
    pub fn SetClipboardText(arg_0: *const c_char);
    pub fn GetClipboardText() -> *const c_char;
    pub fn GetClipboardImage() -> C_Image;
    pub fn EnableEventWaiting();
    pub fn DisableEventWaiting();
    pub fn ShowCursor();
    pub fn HideCursor();
    pub fn IsCursorHidden() -> bool;
    pub fn EnableCursor();
    pub fn DisableCursor();
    pub fn IsCursorOnScreen() -> bool;
    pub fn ClearBackground(arg_0: C_Color);
    pub fn BeginDrawing();
    pub fn EndDrawing();
    pub fn BeginMode2D(arg_0: C_Camera2D);
    pub fn EndMode2D();
    pub fn BeginMode3D(arg_0: C_Camera3D);
    pub fn EndMode3D();
    pub fn BeginTextureMode(arg_0: C_RenderTexture2D);
    pub fn EndTextureMode();
    pub fn BeginShaderMode(arg_0: C_Shader);
    pub fn EndShaderMode();
    pub fn BeginBlendMode(arg_0: c_int);
    pub fn EndBlendMode();
    pub fn BeginScissorMode(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int);
    pub fn EndScissorMode();
    pub fn BeginVrStereoMode(arg_0: C_VrStereoConfig);
    pub fn EndVrStereoMode();
    pub fn LoadVrStereoConfig(arg_0: C_VrDeviceInfo) -> C_VrStereoConfig;
    pub fn UnloadVrStereoConfig(arg_0: C_VrStereoConfig);
    pub fn LoadShader(arg_0: *const c_char, arg_1: *const c_char) -> C_Shader;
    pub fn LoadShaderFromMemory(arg_0: *const c_char, arg_1: *const c_char) -> C_Shader;
    pub fn IsShaderValid(arg_0: C_Shader) -> bool;
    pub fn GetShaderLocation(arg_0: C_Shader, arg_1: *const c_char) -> c_int;
    pub fn GetShaderLocationAttrib(arg_0: C_Shader, arg_1: *const c_char) -> c_int;
    pub fn SetShaderValue(arg_0: C_Shader, arg_1: c_int, arg_2: *mut c_void, arg_3: c_int);
    pub fn SetShaderValueV(arg_0: C_Shader, arg_1: c_int, arg_2: *mut c_void, arg_3: c_int, arg_4: c_int);
    pub fn SetShaderValueMatrix(arg_0: C_Shader, arg_1: c_int, arg_2: C_Matrix);
    pub fn SetShaderValueTexture(arg_0: C_Shader, arg_1: c_int, arg_2: C_Texture2D);
    pub fn UnloadShader(arg_0: C_Shader);
    pub fn GetScreenToWorldRay(arg_0: C_Vector2, arg_1: C_Camera) -> C_Ray;
    pub fn GetScreenToWorldRayEx(arg_0: C_Vector2, arg_1: C_Camera, arg_2: c_int, arg_3: c_int) -> C_Ray;
    pub fn GetWorldToScreen(arg_0: C_Vector3, arg_1: C_Camera) -> C_Vector2;
    pub fn GetWorldToScreenEx(arg_0: C_Vector3, arg_1: C_Camera, arg_2: c_int, arg_3: c_int) -> C_Vector2;
    pub fn GetWorldToScreen2D(arg_0: C_Vector2, arg_1: C_Camera2D) -> C_Vector2;
    pub fn GetScreenToWorld2D(arg_0: C_Vector2, arg_1: C_Camera2D) -> C_Vector2;
    pub fn GetCameraMatrix(arg_0: C_Camera) -> C_Matrix;
    pub fn GetCameraMatrix2D(arg_0: C_Camera2D) -> C_Matrix;
    pub fn SetTargetFPS(arg_0: c_int);
    pub fn GetFrameTime() -> f32;
    pub fn GetTime() -> f64;
    pub fn GetFPS() -> c_int;
    pub fn SwapScreenBuffer();
    pub fn PollInputEvents();
    pub fn WaitTime(arg_0: f64);
    pub fn SetRandomSeed(arg_0: c_uint);
    pub fn GetRandomValue(arg_0: c_int, arg_1: c_int) -> c_int;
    pub fn LoadRandomSequence(arg_0: c_uint, arg_1: c_int, arg_2: c_int) -> *mut c_int;
    pub fn UnloadRandomSequence(arg_0: *mut c_int);
    pub fn TakeScreenshot(arg_0: *const c_char);
    pub fn SetConfigFlags(arg_0: c_uint);
    pub fn OpenURL(arg_0: *const c_char);
    pub fn SetTraceLogLevel(arg_0: c_int);
    pub fn TraceLog(arg_0: c_int, arg_1: *const c_char, ...);
    pub fn SetTraceLogCallback(arg_0: TraceLogCallback);
    pub fn MemAlloc(arg_0: c_uint) -> *mut c_void;
    pub fn MemRealloc(arg_0: *mut c_void, arg_1: c_uint) -> *mut c_void;
    pub fn MemFree(arg_0: *mut c_void);
    pub fn LoadFileData(arg_0: *const c_char, arg_1: *mut c_int) -> *mut u8;
    pub fn UnloadFileData(arg_0: *mut u8);
    pub fn SaveFileData(arg_0: *const c_char, arg_1: *mut c_void, arg_2: c_int) -> bool;
    pub fn ExportDataAsCode(arg_0: *mut u8, arg_1: c_int, arg_2: *const c_char) -> bool;
    pub fn LoadFileText(arg_0: *const c_char) -> *const c_char;
    pub fn UnloadFileText(arg_0: *const c_char);
    pub fn SaveFileText(arg_0: *const c_char, arg_1: *const c_char) -> bool;
    pub fn SetLoadFileDataCallback(arg_0: LoadFileDataCallback);
    pub fn SetSaveFileDataCallback(arg_0: SaveFileDataCallback);
    pub fn SetLoadFileTextCallback(arg_0: LoadFileTextCallback);
    pub fn SetSaveFileTextCallback(arg_0: SaveFileTextCallback);
    pub fn FileRename(arg_0: *const c_char, arg_1: *const c_char) -> c_int;
    pub fn FileRemove(arg_0: *const c_char) -> c_int;
    pub fn FileCopy(arg_0: *const c_char, arg_1: *const c_char) -> c_int;
    pub fn FileMove(arg_0: *const c_char, arg_1: *const c_char) -> c_int;
    pub fn FileTextReplace(arg_0: *const c_char, arg_1: *const c_char, arg_2: *const c_char) -> c_int;
    pub fn FileTextFindIndex(arg_0: *const c_char, arg_1: *const c_char) -> c_int;
    pub fn FileExists(arg_0: *const c_char) -> bool;
    pub fn DirectoryExists(arg_0: *const c_char) -> bool;
    pub fn IsFileExtension(arg_0: *const c_char, arg_1: *const c_char) -> bool;
    pub fn IsFileHidden(arg_0: *const c_char) -> bool;
    pub fn GetFileLength(arg_0: *const c_char) -> c_int;
    pub fn GetFileModTime(arg_0: *const c_char) -> c_longlong;
    pub fn GetFileExtension(arg_0: *const c_char) -> *const c_char;
    pub fn GetFileName(arg_0: *const c_char) -> *const c_char;
    pub fn GetFileNameWithoutExt(arg_0: *const c_char) -> *const c_char;
    pub fn GetDirectoryPath(arg_0: *const c_char) -> *const c_char;
    pub fn GetPrevDirectoryPath(arg_0: *const c_char) -> *const c_char;
    pub fn GetWorkingDirectory() -> *const c_char;
    pub fn GetApplicationDirectory() -> *const c_char;
    pub fn MakeDirectory(arg_0: *const c_char) -> c_int;
    pub fn ChangeDirectory(arg_0: *const c_char) -> c_int;
    pub fn IsPathFile(arg_0: *const c_char) -> bool;
    pub fn IsPathDirectory(arg_0: *const c_char) -> bool;
    pub fn IsPathAbsolute(arg_0: *const c_char) -> bool;
    pub fn IsFileNameValid(arg_0: *const c_char) -> bool;
    pub fn LoadDirectoryFiles(arg_0: *const c_char) -> C_FilePathList;
    pub fn LoadDirectoryFilesEx(arg_0: *const c_char, arg_1: *const c_char, arg_2: bool) -> C_FilePathList;
    pub fn UnloadDirectoryFiles(arg_0: C_FilePathList);
    pub fn IsFileDropped() -> bool;
    pub fn LoadDroppedFiles() -> C_FilePathList;
    pub fn UnloadDroppedFiles(arg_0: C_FilePathList);
    pub fn GetDirectoryFileCount(arg_0: *const c_char) -> c_uint;
    pub fn GetDirectoryFileCountEx(arg_0: *const c_char, arg_1: *const c_char, arg_2: bool) -> c_uint;
    pub fn CompressData(arg_0: *mut u8, arg_1: c_int, arg_2: *mut c_int) -> *mut u8;
    pub fn DecompressData(arg_0: *mut u8, arg_1: c_int, arg_2: *mut c_int) -> *mut u8;
    pub fn EncodeDataBase64(arg_0: *mut u8, arg_1: c_int, arg_2: *mut c_int) -> *const c_char;
    pub fn DecodeDataBase64(arg_0: *const c_char, arg_1: *mut c_int) -> *mut u8;
    pub fn ComputeCRC32(arg_0: *mut u8, arg_1: c_int) -> c_uint;
    pub fn ComputeMD5(arg_0: *mut u8, arg_1: c_int) -> *mut c_uint;
    pub fn ComputeSHA1(arg_0: *mut u8, arg_1: c_int) -> *mut c_uint;
    pub fn ComputeSHA256(arg_0: *mut u8, arg_1: c_int) -> *mut c_uint;
    pub fn LoadAutomationEventList(arg_0: *const c_char) -> C_AutomationEventList;
    pub fn UnloadAutomationEventList(arg_0: C_AutomationEventList);
    pub fn ExportAutomationEventList(arg_0: C_AutomationEventList, arg_1: *const c_char) -> bool;
    pub fn SetAutomationEventList(arg_0: *mut C_AutomationEventList);
    pub fn SetAutomationEventBaseFrame(arg_0: c_int);
    pub fn StartAutomationEventRecording();
    pub fn StopAutomationEventRecording();
    pub fn PlayAutomationEvent(arg_0: C_AutomationEvent);
    pub fn IsKeyPressed(arg_0: c_int) -> bool;
    pub fn IsKeyPressedRepeat(arg_0: c_int) -> bool;
    pub fn IsKeyDown(arg_0: c_int) -> bool;
    pub fn IsKeyReleased(arg_0: c_int) -> bool;
    pub fn IsKeyUp(arg_0: c_int) -> bool;
    pub fn GetKeyPressed() -> c_int;
    pub fn GetCharPressed() -> c_int;
    pub fn GetKeyName(arg_0: c_int) -> *const c_char;
    pub fn SetExitKey(arg_0: c_int);
    pub fn IsGamepadAvailable(arg_0: c_int) -> bool;
    pub fn GetGamepadName(arg_0: c_int) -> *const c_char;
    pub fn IsGamepadButtonPressed(arg_0: c_int, arg_1: c_int) -> bool;
    pub fn IsGamepadButtonDown(arg_0: c_int, arg_1: c_int) -> bool;
    pub fn IsGamepadButtonReleased(arg_0: c_int, arg_1: c_int) -> bool;
    pub fn IsGamepadButtonUp(arg_0: c_int, arg_1: c_int) -> bool;
    pub fn GetGamepadButtonPressed() -> c_int;
    pub fn GetGamepadAxisCount(arg_0: c_int) -> c_int;
    pub fn GetGamepadAxisMovement(arg_0: c_int, arg_1: c_int) -> f32;
    pub fn SetGamepadMappings(arg_0: *const c_char) -> c_int;
    pub fn SetGamepadVibration(arg_0: c_int, arg_1: f32, arg_2: f32, arg_3: f32);
    pub fn IsMouseButtonPressed(arg_0: c_int) -> bool;
    pub fn IsMouseButtonDown(arg_0: c_int) -> bool;
    pub fn IsMouseButtonReleased(arg_0: c_int) -> bool;
    pub fn IsMouseButtonUp(arg_0: c_int) -> bool;
    pub fn GetMouseX() -> c_int;
    pub fn GetMouseY() -> c_int;
    pub fn GetMousePosition() -> C_Vector2;
    pub fn GetMouseDelta() -> C_Vector2;
    pub fn SetMousePosition(arg_0: c_int, arg_1: c_int);
    pub fn SetMouseOffset(arg_0: c_int, arg_1: c_int);
    pub fn SetMouseScale(arg_0: f32, arg_1: f32);
    pub fn GetMouseWheelMove() -> f32;
    pub fn GetMouseWheelMoveV() -> C_Vector2;
    pub fn SetMouseCursor(arg_0: c_int);
    pub fn GetTouchX() -> c_int;
    pub fn GetTouchY() -> c_int;
    pub fn GetTouchPosition(arg_0: c_int) -> C_Vector2;
    pub fn GetTouchPointId(arg_0: c_int) -> c_int;
    pub fn GetTouchPointCount() -> c_int;
    pub fn SetGesturesEnabled(arg_0: c_uint);
    pub fn IsGestureDetected(arg_0: c_uint) -> bool;
    pub fn GetGestureDetected() -> c_int;
    pub fn GetGestureHoldDuration() -> f32;
    pub fn GetGestureDragVector() -> C_Vector2;
    pub fn GetGestureDragAngle() -> f32;
    pub fn GetGesturePinchVector() -> C_Vector2;
    pub fn GetGesturePinchAngle() -> f32;
    pub fn UpdateCamera(arg_0: *mut C_Camera, arg_1: c_int);
    pub fn UpdateCameraPro(arg_0: *mut C_Camera, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: f32);
    pub fn SetShapesTexture(arg_0: C_Texture2D, arg_1: C_Rectangle);
    pub fn GetShapesTexture() -> C_Texture2D;
    pub fn GetShapesTextureRectangle() -> C_Rectangle;
    pub fn DrawPixel(arg_0: c_int, arg_1: c_int, arg_2: C_Color);
    pub fn DrawPixelV(arg_0: C_Vector2, arg_1: C_Color);
    pub fn DrawLine(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawLineV(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Color);
    pub fn DrawLineEx(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32, arg_3: C_Color);
    pub fn DrawLineStrip(arg_0: *const C_Vector2, arg_1: c_int, arg_2: C_Color);
    pub fn DrawLineBezier(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32, arg_3: C_Color);
    pub fn DrawLineDashed(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawTriangle(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Color);
    pub fn DrawTriangleGradient(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Color, arg_4: C_Color, arg_5: C_Color);
    pub fn DrawTriangleLines(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Color);
    pub fn DrawTriangleLinesEx(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: f32, arg_4: C_Color);
    pub fn DrawTriangleFan(arg_0: *const C_Vector2, arg_1: c_int, arg_2: C_Color);
    pub fn DrawTriangleStrip(arg_0: *const C_Vector2, arg_1: c_int, arg_2: C_Color);
    pub fn DrawRectangle(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawRectangleV(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Color);
    pub fn DrawRectangleRec(arg_0: C_Rectangle, arg_1: C_Color);
    pub fn DrawRectanglePro(arg_0: C_Rectangle, arg_1: C_Vector2, arg_2: f32, arg_3: C_Color);
    pub fn DrawRectangleGradientV(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color, arg_5: C_Color);
    pub fn DrawRectangleGradientH(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color, arg_5: C_Color);
    pub fn DrawRectangleGradientEx(arg_0: C_Rectangle, arg_1: C_Color, arg_2: C_Color, arg_3: C_Color, arg_4: C_Color);
    pub fn DrawRectangleLines(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawRectangleLinesEx(arg_0: C_Rectangle, arg_1: f32, arg_2: C_Color);
    pub fn DrawRectangleRounded(arg_0: C_Rectangle, arg_1: f32, arg_2: c_int, arg_3: C_Color);
    pub fn DrawRectangleRoundedLines(arg_0: C_Rectangle, arg_1: f32, arg_2: c_int, arg_3: C_Color);
    pub fn DrawRectangleRoundedLinesEx(arg_0: C_Rectangle, arg_1: f32, arg_2: c_int, arg_3: f32, arg_4: C_Color);
    pub fn DrawPoly(arg_0: C_Vector2, arg_1: c_int, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawPolyLines(arg_0: C_Vector2, arg_1: c_int, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawPolyLinesEx(arg_0: C_Vector2, arg_1: c_int, arg_2: f32, arg_3: f32, arg_4: f32, arg_5: C_Color);
    pub fn DrawCircle(arg_0: c_int, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawCircleV(arg_0: C_Vector2, arg_1: f32, arg_2: C_Color);
    pub fn DrawCircleGradient(arg_0: C_Vector2, arg_1: f32, arg_2: C_Color, arg_3: C_Color);
    pub fn DrawCircleSector(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCircleSectorLines(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCircleSectorLinesEx(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: f32, arg_6: C_Color);
    pub fn DrawCircleLines(arg_0: c_int, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawCircleLinesV(arg_0: C_Vector2, arg_1: f32, arg_2: C_Color);
    pub fn DrawCircleLinesEx(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: C_Color);
    pub fn DrawEllipse(arg_0: c_int, arg_1: c_int, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawEllipseV(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: C_Color);
    pub fn DrawEllipseLines(arg_0: c_int, arg_1: c_int, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawEllipseLinesV(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: C_Color);
    pub fn DrawEllipseLinesEx(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawRing(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: f32, arg_5: c_int, arg_6: C_Color);
    pub fn DrawRingLines(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: f32, arg_5: c_int, arg_6: C_Color);
    pub fn DrawRingLinesEx(arg_0: C_Vector2, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: f32, arg_5: c_int, arg_6: f32, arg_7: C_Color);
    pub fn DrawSplineLinear(arg_0: *const C_Vector2, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawSplineBasis(arg_0: *const C_Vector2, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawSplineCatmullRom(arg_0: *const C_Vector2, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawSplineBezierQuadratic(arg_0: *const C_Vector2, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawSplineBezierCubic(arg_0: *const C_Vector2, arg_1: c_int, arg_2: f32, arg_3: C_Color);
    pub fn DrawSplineSegmentLinear(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32, arg_3: C_Color);
    pub fn DrawSplineSegmentBasis(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
    pub fn DrawSplineSegmentCatmullRom(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
    pub fn DrawSplineSegmentBezierQuadratic(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: f32, arg_4: C_Color);
    pub fn DrawSplineSegmentBezierCubic(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
    pub fn GetSplinePointLinear(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32) -> C_Vector2;
    pub fn GetSplinePointBasis(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32) -> C_Vector2;
    pub fn GetSplinePointCatmullRom(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32) -> C_Vector2;
    pub fn GetSplinePointBezierQuadratic(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: f32) -> C_Vector2;
    pub fn GetSplinePointBezierCubic(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32) -> C_Vector2;
    pub fn CheckCollisionRecs(arg_0: C_Rectangle, arg_1: C_Rectangle) -> bool;
    pub fn CheckCollisionCircles(arg_0: C_Vector2, arg_1: f32, arg_2: C_Vector2, arg_3: f32) -> bool;
    pub fn CheckCollisionCircleRec(arg_0: C_Vector2, arg_1: f32, arg_2: C_Rectangle) -> bool;
    pub fn CheckCollisionCircleLine(arg_0: C_Vector2, arg_1: f32, arg_2: C_Vector2, arg_3: C_Vector2) -> bool;
    pub fn CheckCollisionPointRec(arg_0: C_Vector2, arg_1: C_Rectangle) -> bool;
    pub fn CheckCollisionPointCircle(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32) -> bool;
    pub fn CheckCollisionPointTriangle(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2) -> bool;
    pub fn CheckCollisionPointLine(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: c_int) -> bool;
    pub fn CheckCollisionPointPoly(arg_0: C_Vector2, arg_1: *const C_Vector2, arg_2: c_int) -> bool;
    pub fn CheckCollisionLines(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: *mut C_Vector2) -> bool;
    pub fn GetCollisionRec(arg_0: C_Rectangle, arg_1: C_Rectangle) -> C_Rectangle;
    pub fn LoadImage(arg_0: *const c_char) -> C_Image;
    pub fn LoadImageRaw(arg_0: *const c_char, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int) -> C_Image;
    pub fn LoadImageAnim(arg_0: *const c_char, arg_1: *mut c_int) -> C_Image;
    pub fn LoadImageAnimFromMemory(arg_0: *const c_char, arg_1: *mut u8, arg_2: c_int, arg_3: *mut c_int) -> C_Image;
    pub fn LoadImageFromMemory(arg_0: *const c_char, arg_1: *mut u8, arg_2: c_int) -> C_Image;
    pub fn LoadImageFromTexture(arg_0: C_Texture2D) -> C_Image;
    pub fn LoadImageFromScreen() -> C_Image;
    pub fn IsImageValid(arg_0: C_Image) -> bool;
    pub fn UnloadImage(arg_0: C_Image);
    pub fn ExportImage(arg_0: C_Image, arg_1: *const c_char) -> bool;
    pub fn ExportImageToMemory(arg_0: C_Image, arg_1: *const c_char, arg_2: *mut c_int) -> *mut u8;
    pub fn ExportImageAsCode(arg_0: C_Image, arg_1: *const c_char) -> bool;
    pub fn GenImageColor(arg_0: c_int, arg_1: c_int, arg_2: C_Color) -> C_Image;
    pub fn GenImageGradientLinear(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: C_Color, arg_4: C_Color) -> C_Image;
    pub fn GenImageGradientRadial(arg_0: c_int, arg_1: c_int, arg_2: f32, arg_3: C_Color, arg_4: C_Color) -> C_Image;
    pub fn GenImageGradientSquare(arg_0: c_int, arg_1: c_int, arg_2: f32, arg_3: C_Color, arg_4: C_Color) -> C_Image;
    pub fn GenImageChecked(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color, arg_5: C_Color) -> C_Image;
    pub fn GenImageWhiteNoise(arg_0: c_int, arg_1: c_int, arg_2: f32) -> C_Image;
    pub fn GenImagePerlinNoise(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: f32) -> C_Image;
    pub fn GenImageCellular(arg_0: c_int, arg_1: c_int, arg_2: c_int) -> C_Image;
    pub fn GenImageText(arg_0: c_int, arg_1: c_int, arg_2: *const c_char) -> C_Image;
    pub fn ImageCopy(arg_0: C_Image) -> C_Image;
    pub fn ImageFromImage(arg_0: C_Image, arg_1: C_Rectangle) -> C_Image;
    pub fn ImageFromChannel(arg_0: C_Image, arg_1: c_int) -> C_Image;
    pub fn ImageText(arg_0: *const c_char, arg_1: c_int, arg_2: C_Color) -> C_Image;
    pub fn ImageTextEx(arg_0: C_Font, arg_1: *const c_char, arg_2: f32, arg_3: f32, arg_4: C_Color) -> C_Image;
    pub fn ImageFormat(arg_0: *mut C_Image, arg_1: c_int);
    pub fn ImageToPOT(arg_0: *mut C_Image, arg_1: C_Color);
    pub fn ImageCrop(arg_0: *mut C_Image, arg_1: C_Rectangle);
    pub fn ImageAlphaCrop(arg_0: *mut C_Image, arg_1: f32);
    pub fn ImageAlphaClear(arg_0: *mut C_Image, arg_1: C_Color, arg_2: f32);
    pub fn ImageAlphaMask(arg_0: *mut C_Image, arg_1: C_Image);
    pub fn ImageAlphaPremultiply(arg_0: *mut C_Image);
    pub fn ImageBlurGaussian(arg_0: *mut C_Image, arg_1: c_int);
    pub fn ImageKernelConvolution(arg_0: *mut C_Image, arg_1: *mut f32, arg_2: c_int);
    pub fn ImageResize(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int);
    pub fn ImageResizeNN(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int);
    pub fn ImageResizeCanvas(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn ImageMipmaps(arg_0: *mut C_Image);
    pub fn ImageDither(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int);
    pub fn ImageFlipVertical(arg_0: *mut C_Image);
    pub fn ImageFlipHorizontal(arg_0: *mut C_Image);
    pub fn ImageRotate(arg_0: *mut C_Image, arg_1: c_int);
    pub fn ImageRotateCW(arg_0: *mut C_Image);
    pub fn ImageRotateCCW(arg_0: *mut C_Image);
    pub fn ImageColorTint(arg_0: *mut C_Image, arg_1: C_Color);
    pub fn ImageColorInvert(arg_0: *mut C_Image);
    pub fn ImageColorGrayscale(arg_0: *mut C_Image);
    pub fn ImageColorContrast(arg_0: *mut C_Image, arg_1: c_int);
    pub fn ImageColorBrightness(arg_0: *mut C_Image, arg_1: c_int);
    pub fn ImageColorReplace(arg_0: *mut C_Image, arg_1: C_Color, arg_2: C_Color);
    pub fn LoadImageColors(arg_0: C_Image) -> *mut C_Color;
    pub fn LoadImagePalette(arg_0: C_Image, arg_1: c_int, arg_2: *mut c_int) -> *mut C_Color;
    pub fn UnloadImageColors(arg_0: *mut C_Color);
    pub fn UnloadImagePalette(arg_0: *mut C_Color);
    pub fn GetImageAlphaBorder(arg_0: C_Image, arg_1: f32) -> C_Rectangle;
    pub fn GetImageColor(arg_0: C_Image, arg_1: c_int, arg_2: c_int) -> C_Color;
    pub fn ImageClearBackground(arg_0: *mut C_Image, arg_1: C_Color);
    pub fn ImageDrawPixel(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawPixelV(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Color);
    pub fn ImageDrawLine(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn ImageDrawLineV(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Color);
    pub fn ImageDrawLineEx(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: c_int, arg_4: C_Color);
    pub fn ImageDrawLineStrip(arg_0: *mut C_Image, arg_1: *const C_Vector2, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawTriangle(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: C_Color);
    pub fn ImageDrawTriangleGradient(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: C_Color, arg_5: C_Color, arg_6: C_Color);
    pub fn ImageDrawTriangleLines(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: C_Color);
    pub fn ImageDrawTriangleFan(arg_0: *mut C_Image, arg_1: *const C_Vector2, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawTriangleStrip(arg_0: *mut C_Image, arg_1: *const C_Vector2, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawRectangle(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn ImageDrawRectangleV(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: C_Vector2, arg_3: C_Color);
    pub fn ImageDrawRectangleRec(arg_0: *mut C_Image, arg_1: C_Rectangle, arg_2: C_Color);
    pub fn ImageDrawRectanglePro(arg_0: *mut C_Image, arg_1: C_Rectangle, arg_2: C_Vector2, arg_3: f32, arg_4: C_Color);
    pub fn ImageDrawRectangleLines(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn ImageDrawRectangleLinesEx(arg_0: *mut C_Image, arg_1: C_Rectangle, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawRectangleGradientEx(arg_0: *mut C_Image, arg_1: C_Rectangle, arg_2: C_Color, arg_3: C_Color, arg_4: C_Color, arg_5: C_Color);
    pub fn ImageDrawCircle(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn ImageDrawCircleV(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawCircleLines(arg_0: *mut C_Image, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn ImageDrawCircleLinesV(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: c_int, arg_3: C_Color);
    pub fn ImageDrawCircleGradient(arg_0: *mut C_Image, arg_1: C_Vector2, arg_2: f32, arg_3: C_Color, arg_4: C_Color);
    pub fn ImageDrawImage(arg_0: *mut C_Image, arg_1: C_Image, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn ImageDrawImageEx(arg_0: *mut C_Image, arg_1: C_Image, arg_2: C_Vector2, arg_3: f32, arg_4: f32, arg_5: C_Color);
    pub fn ImageDrawImageRec(arg_0: *mut C_Image, arg_1: C_Image, arg_2: C_Rectangle, arg_3: C_Vector2, arg_4: C_Color);
    pub fn ImageDrawImagePro(arg_0: *mut C_Image, arg_1: C_Image, arg_2: C_Rectangle, arg_3: C_Rectangle, arg_4: C_Vector2, arg_5: f32, arg_6: C_Color);
    pub fn ImageDrawText(arg_0: *mut C_Image, arg_1: *const c_char, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn ImageDrawTextEx(arg_0: *mut C_Image, arg_1: C_Font, arg_2: *const c_char, arg_3: C_Vector2, arg_4: f32, arg_5: f32, arg_6: C_Color);
    pub fn ImageDrawTextPro(arg_0: *mut C_Image, arg_1: C_Font, arg_2: *const c_char, arg_3: C_Vector2, arg_4: C_Vector2, arg_5: f32, arg_6: f32, arg_7: f32, arg_8: C_Color);
    pub fn LoadTexture(arg_0: *const c_char) -> C_Texture2D;
    pub fn LoadTextureFromImage(arg_0: C_Image) -> C_Texture2D;
    pub fn LoadTextureCubemap(arg_0: C_Image, arg_1: c_int) -> C_TextureCubemap;
    pub fn LoadRenderTexture(arg_0: c_int, arg_1: c_int) -> C_RenderTexture2D;
    pub fn LoadRenderTextureEx(arg_0: c_int, arg_1: c_int, arg_2: c_int) -> C_RenderTexture2D;
    pub fn IsTextureValid(arg_0: C_Texture2D) -> bool;
    pub fn UnloadTexture(arg_0: C_Texture2D);
    pub fn IsRenderTextureValid(arg_0: C_RenderTexture2D) -> bool;
    pub fn UnloadRenderTexture(arg_0: C_RenderTexture2D);
    pub fn UpdateTexture(arg_0: C_Texture2D, arg_1: *mut c_void);
    pub fn UpdateTextureRec(arg_0: C_Texture2D, arg_1: C_Rectangle, arg_2: *mut c_void);
    pub fn GenTextureMipmaps(arg_0: *mut C_Texture2D);
    pub fn SetTextureFilter(arg_0: C_Texture2D, arg_1: c_int);
    pub fn SetTextureWrap(arg_0: C_Texture2D, arg_1: c_int);
    pub fn DrawTexture(arg_0: C_Texture2D, arg_1: c_int, arg_2: c_int, arg_3: C_Color);
    pub fn DrawTextureV(arg_0: C_Texture2D, arg_1: C_Vector2, arg_2: C_Color);
    pub fn DrawTextureEx(arg_0: C_Texture2D, arg_1: C_Vector2, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawTextureRec(arg_0: C_Texture2D, arg_1: C_Rectangle, arg_2: C_Vector2, arg_3: C_Color);
    pub fn DrawTexturePro(arg_0: C_Texture2D, arg_1: C_Rectangle, arg_2: C_Rectangle, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
    pub fn DrawTextureNPatch(arg_0: C_Texture2D, arg_1: C_NPatchInfo, arg_2: C_Rectangle, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
    pub fn ColorIsEqual(arg_0: C_Color, arg_1: C_Color) -> bool;
    pub fn Fade(arg_0: C_Color, arg_1: f32) -> C_Color;
    pub fn ColorToInt(arg_0: C_Color) -> c_int;
    pub fn ColorNormalize(arg_0: C_Color) -> C_Vector4;
    pub fn ColorFromNormalized(arg_0: C_Vector4) -> C_Color;
    pub fn ColorToHSV(arg_0: C_Color) -> C_Vector3;
    pub fn ColorFromHSV(arg_0: f32, arg_1: f32, arg_2: f32) -> C_Color;
    pub fn ColorTint(arg_0: C_Color, arg_1: C_Color) -> C_Color;
    pub fn ColorBrightness(arg_0: C_Color, arg_1: f32) -> C_Color;
    pub fn ColorContrast(arg_0: C_Color, arg_1: f32) -> C_Color;
    pub fn ColorAlpha(arg_0: C_Color, arg_1: f32) -> C_Color;
    pub fn ColorAlphaBlend(arg_0: C_Color, arg_1: C_Color, arg_2: C_Color) -> C_Color;
    pub fn ColorLerp(arg_0: C_Color, arg_1: C_Color, arg_2: f32) -> C_Color;
    pub fn GetColor(arg_0: c_uint) -> C_Color;
    pub fn GetPixelColor(arg_0: *mut c_void, arg_1: c_int) -> C_Color;
    pub fn SetPixelColor(arg_0: *mut c_void, arg_1: C_Color, arg_2: c_int);
    pub fn GetPixelDataSize(arg_0: c_int, arg_1: c_int, arg_2: c_int) -> c_int;
    pub fn GetFontDefault() -> C_Font;
    pub fn LoadFont(arg_0: *const c_char) -> C_Font;
    pub fn LoadFontEx(arg_0: *const c_char, arg_1: c_int, arg_2: *mut c_int, arg_3: c_int) -> C_Font;
    pub fn LoadFontFromImage(arg_0: C_Image, arg_1: C_Color, arg_2: c_int) -> C_Font;
    pub fn LoadFontFromMemory(arg_0: *const c_char, arg_1: *mut u8, arg_2: c_int, arg_3: c_int, arg_4: *mut c_int, arg_5: c_int) -> C_Font;
    pub fn IsFontValid(arg_0: C_Font) -> bool;
    pub fn LoadFontData(arg_0: *mut u8, arg_1: c_int, arg_2: c_int, arg_3: *mut c_int, arg_4: c_int, arg_5: c_int, arg_6: *mut c_int) -> *mut C_GlyphInfo;
    pub fn GenImageFontAtlas(arg_0: *const C_GlyphInfo, arg_1: *mut *mut C_Rectangle, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: c_int) -> C_Image;
    pub fn UnloadFontData(arg_0: *mut C_GlyphInfo, arg_1: c_int);
    pub fn UnloadFont(arg_0: C_Font);
    pub fn ExportFontAsCode(arg_0: C_Font, arg_1: *const c_char) -> bool;
    pub fn DrawFPS(arg_0: c_int, arg_1: c_int);
    pub fn DrawText(arg_0: *const c_char, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawTextEx(arg_0: C_Font, arg_1: *const c_char, arg_2: C_Vector2, arg_3: f32, arg_4: f32, arg_5: C_Color);
    pub fn DrawTextPro(arg_0: C_Font, arg_1: *const c_char, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32, arg_5: f32, arg_6: f32, arg_7: C_Color);
    pub fn DrawTextCodepoint(arg_0: C_Font, arg_1: c_int, arg_2: C_Vector2, arg_3: f32, arg_4: C_Color);
    pub fn DrawTextCodepoints(arg_0: C_Font, arg_1: *mut c_int, arg_2: c_int, arg_3: C_Vector2, arg_4: f32, arg_5: f32, arg_6: C_Color);
    pub fn SetTextLineSpacing(arg_0: c_int);
    pub fn MeasureText(arg_0: *const c_char, arg_1: c_int) -> c_int;
    pub fn MeasureTextEx(arg_0: C_Font, arg_1: *const c_char, arg_2: f32, arg_3: f32) -> C_Vector2;
    pub fn MeasureTextCodepoints(arg_0: C_Font, arg_1: *mut c_int, arg_2: c_int, arg_3: f32, arg_4: f32) -> C_Vector2;
    pub fn GetGlyphIndex(arg_0: C_Font, arg_1: c_int) -> c_int;
    pub fn GetGlyphInfo(arg_0: C_Font, arg_1: c_int) -> C_GlyphInfo;
    pub fn GetGlyphAtlasRec(arg_0: C_Font, arg_1: c_int) -> C_Rectangle;
    pub fn LoadUTF8(arg_0: *mut c_int, arg_1: c_int) -> *const c_char;
    pub fn UnloadUTF8(arg_0: *const c_char);
    pub fn LoadCodepoints(arg_0: *const c_char, arg_1: *mut c_int) -> *mut c_int;
    pub fn UnloadCodepoints(arg_0: *mut c_int);
    pub fn GetCodepointCount(arg_0: *const c_char) -> c_int;
    pub fn GetCodepoint(arg_0: *const c_char, arg_1: *mut c_int) -> c_int;
    pub fn GetCodepointNext(arg_0: *const c_char, arg_1: *mut c_int) -> c_int;
    pub fn GetCodepointPrevious(arg_0: *const c_char, arg_1: *mut c_int) -> c_int;
    pub fn CodepointToUTF8(arg_0: c_int, arg_1: *mut c_int) -> *const c_char;
    pub fn LoadTextLines(arg_0: *const c_char, arg_1: *mut c_int) -> *mut *mut c_char;
    pub fn UnloadTextLines(arg_0: *mut *mut c_char, arg_1: c_int);
    pub fn TextCopy(arg_0: *const c_char, arg_1: *const c_char) -> c_int;
    pub fn TextIsEqual(arg_0: *const c_char, arg_1: *const c_char) -> bool;
    pub fn TextLength(arg_0: *const c_char) -> c_uint;
    pub fn TextFormat(arg_0: *const c_char, ...) -> *const c_char;
    pub fn TextSubtext(arg_0: *const c_char, arg_1: c_int, arg_2: c_int) -> *const c_char;
    pub fn TextRemoveSpaces(arg_0: *const c_char) -> *const c_char;
    pub fn GetTextBetween(arg_0: *const c_char, arg_1: *const c_char, arg_2: *const c_char) -> *const c_char;
    pub fn TextReplace(arg_0: *const c_char, arg_1: *const c_char, arg_2: *const c_char) -> *const c_char;
    pub fn TextReplaceAlloc(arg_0: *const c_char, arg_1: *const c_char, arg_2: *const c_char) -> *const c_char;
    pub fn TextReplaceBetween(arg_0: *const c_char, arg_1: *const c_char, arg_2: *const c_char, arg_3: *const c_char) -> *const c_char;
    pub fn TextReplaceBetweenAlloc(arg_0: *const c_char, arg_1: *const c_char, arg_2: *const c_char, arg_3: *const c_char) -> *const c_char;
    pub fn TextInsert(arg_0: *const c_char, arg_1: *const c_char, arg_2: c_int) -> *const c_char;
    pub fn TextInsertAlloc(arg_0: *const c_char, arg_1: *const c_char, arg_2: c_int) -> *const c_char;
    pub fn TextJoin(arg_0: *mut *mut c_char, arg_1: c_int, arg_2: *const c_char) -> *const c_char;
    pub fn TextSplit(arg_0: *const c_char, arg_1: c_char, arg_2: *mut c_int) -> *mut *mut c_char;
    pub fn TextAppend(arg_0: *const c_char, arg_1: *const c_char, arg_2: *mut c_int);
    pub fn TextFindIndex(arg_0: *const c_char, arg_1: *const c_char) -> c_int;
    pub fn TextToUpper(arg_0: *const c_char) -> *const c_char;
    pub fn TextToLower(arg_0: *const c_char) -> *const c_char;
    pub fn TextToPascal(arg_0: *const c_char) -> *const c_char;
    pub fn TextToSnake(arg_0: *const c_char) -> *const c_char;
    pub fn TextToCamel(arg_0: *const c_char) -> *const c_char;
    pub fn TextToInteger(arg_0: *const c_char) -> c_int;
    pub fn TextToFloat(arg_0: *const c_char) -> f32;
    pub fn DrawLine3D(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Color);
    pub fn DrawPoint3D(arg_0: C_Vector3, arg_1: C_Color);
    pub fn DrawCircle3D(arg_0: C_Vector3, arg_1: f32, arg_2: C_Vector3, arg_3: f32, arg_4: C_Color);
    pub fn DrawTriangle3D(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: C_Color);
    pub fn DrawTriangleStrip3D(arg_0: *const C_Vector3, arg_1: c_int, arg_2: C_Color);
    pub fn DrawCube(arg_0: C_Vector3, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawCubeV(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Color);
    pub fn DrawCubeWires(arg_0: C_Vector3, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawCubeWiresV(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Color);
    pub fn DrawSphere(arg_0: C_Vector3, arg_1: f32, arg_2: C_Color);
    pub fn DrawSphereEx(arg_0: C_Vector3, arg_1: f32, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawSphereWires(arg_0: C_Vector3, arg_1: f32, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawCylinder(arg_0: C_Vector3, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCylinderEx(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCylinderWires(arg_0: C_Vector3, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCylinderWiresEx(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32, arg_3: f32, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCapsule(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn DrawCapsuleWires(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32, arg_3: c_int, arg_4: c_int, arg_5: C_Color);
    pub fn DrawPlane(arg_0: C_Vector3, arg_1: C_Vector2, arg_2: C_Color);
    pub fn DrawRay(arg_0: C_Ray, arg_1: C_Color);
    pub fn DrawGrid(arg_0: c_int, arg_1: f32);
    pub fn LoadModel(arg_0: *const c_char) -> C_Model;
    pub fn LoadModelFromMesh(arg_0: C_Mesh) -> C_Model;
    pub fn IsModelValid(arg_0: C_Model) -> bool;
    pub fn UnloadModel(arg_0: C_Model);
    pub fn GetModelBoundingBox(arg_0: C_Model) -> C_BoundingBox;
    pub fn DrawModel(arg_0: C_Model, arg_1: C_Vector3, arg_2: f32, arg_3: C_Color);
    pub fn DrawModelEx(arg_0: C_Model, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: f32, arg_4: C_Vector3, arg_5: C_Color);
    pub fn DrawModelWires(arg_0: C_Model, arg_1: C_Vector3, arg_2: f32, arg_3: C_Color);
    pub fn DrawModelWiresEx(arg_0: C_Model, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: f32, arg_4: C_Vector3, arg_5: C_Color);
    pub fn DrawBoundingBox(arg_0: C_BoundingBox, arg_1: C_Color);
    pub fn DrawBillboard(arg_0: C_Camera, arg_1: C_Texture2D, arg_2: C_Vector3, arg_3: f32, arg_4: C_Color);
    pub fn DrawBillboardRec(arg_0: C_Camera, arg_1: C_Texture2D, arg_2: C_Rectangle, arg_3: C_Vector3, arg_4: C_Vector2, arg_5: C_Color);
    pub fn DrawBillboardPro(arg_0: C_Camera, arg_1: C_Texture2D, arg_2: C_Rectangle, arg_3: C_Vector3, arg_4: C_Vector3, arg_5: C_Vector2, arg_6: C_Vector2, arg_7: f32, arg_8: C_Color);
    pub fn UploadMesh(arg_0: *mut C_Mesh, arg_1: bool);
    pub fn UpdateMeshBuffer(arg_0: C_Mesh, arg_1: c_int, arg_2: *mut c_void, arg_3: c_int, arg_4: c_int);
    pub fn UnloadMesh(arg_0: C_Mesh);
    pub fn DrawMesh(arg_0: C_Mesh, arg_1: C_Material, arg_2: C_Matrix);
    pub fn DrawMeshInstanced(arg_0: C_Mesh, arg_1: C_Material, arg_2: *const C_Matrix, arg_3: c_int);
    pub fn GetMeshBoundingBox(arg_0: C_Mesh) -> C_BoundingBox;
    pub fn GenMeshTangents(arg_0: *mut C_Mesh);
    pub fn ExportMesh(arg_0: C_Mesh, arg_1: *const c_char) -> bool;
    pub fn ExportMeshAsCode(arg_0: C_Mesh, arg_1: *const c_char) -> bool;
    pub fn GenMeshPoly(arg_0: c_int, arg_1: f32) -> C_Mesh;
    pub fn GenMeshPlane(arg_0: f32, arg_1: f32, arg_2: c_int, arg_3: c_int) -> C_Mesh;
    pub fn GenMeshCube(arg_0: f32, arg_1: f32, arg_2: f32) -> C_Mesh;
    pub fn GenMeshSphere(arg_0: f32, arg_1: c_int, arg_2: c_int) -> C_Mesh;
    pub fn GenMeshHemiSphere(arg_0: f32, arg_1: c_int, arg_2: c_int) -> C_Mesh;
    pub fn GenMeshCylinder(arg_0: f32, arg_1: f32, arg_2: c_int) -> C_Mesh;
    pub fn GenMeshCone(arg_0: f32, arg_1: f32, arg_2: c_int) -> C_Mesh;
    pub fn GenMeshTorus(arg_0: f32, arg_1: f32, arg_2: c_int, arg_3: c_int) -> C_Mesh;
    pub fn GenMeshKnot(arg_0: f32, arg_1: f32, arg_2: c_int, arg_3: c_int) -> C_Mesh;
    pub fn GenMeshHeightmap(arg_0: C_Image, arg_1: C_Vector3) -> C_Mesh;
    pub fn GenMeshCubicmap(arg_0: C_Image, arg_1: C_Vector3) -> C_Mesh;
    pub fn LoadMaterials(arg_0: *const c_char, arg_1: *mut c_int) -> *mut C_Material;
    pub fn LoadMaterialDefault() -> C_Material;
    pub fn IsMaterialValid(arg_0: C_Material) -> bool;
    pub fn UnloadMaterial(arg_0: C_Material);
    pub fn SetMaterialTexture(arg_0: *mut C_Material, arg_1: c_int, arg_2: C_Texture2D);
    pub fn SetModelMeshMaterial(arg_0: *mut C_Model, arg_1: c_int, arg_2: c_int);
    pub fn LoadModelAnimations(arg_0: *const c_char, arg_1: *mut c_int) -> *mut C_ModelAnimation;
    pub fn UpdateModelAnimation(arg_0: C_Model, arg_1: C_ModelAnimation, arg_2: f32);
    pub fn UpdateModelAnimationEx(arg_0: C_Model, arg_1: C_ModelAnimation, arg_2: f32, arg_3: C_ModelAnimation, arg_4: f32, arg_5: f32);
    pub fn UnloadModelAnimations(arg_0: *mut C_ModelAnimation, arg_1: c_int);
    pub fn IsModelAnimationValid(arg_0: C_Model, arg_1: C_ModelAnimation) -> bool;
    pub fn CheckCollisionSpheres(arg_0: C_Vector3, arg_1: f32, arg_2: C_Vector3, arg_3: f32) -> bool;
    pub fn CheckCollisionBoxes(arg_0: C_BoundingBox, arg_1: C_BoundingBox) -> bool;
    pub fn CheckCollisionBoxSphere(arg_0: C_BoundingBox, arg_1: C_Vector3, arg_2: f32) -> bool;
    pub fn GetRayCollisionSphere(arg_0: C_Ray, arg_1: C_Vector3, arg_2: f32) -> C_RayCollision;
    pub fn GetRayCollisionBox(arg_0: C_Ray, arg_1: C_BoundingBox) -> C_RayCollision;
    pub fn GetRayCollisionMesh(arg_0: C_Ray, arg_1: C_Mesh, arg_2: C_Matrix) -> C_RayCollision;
    pub fn GetRayCollisionTriangle(arg_0: C_Ray, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: C_Vector3) -> C_RayCollision;
    pub fn GetRayCollisionQuad(arg_0: C_Ray, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: C_Vector3, arg_4: C_Vector3) -> C_RayCollision;
    pub fn InitAudioDevice();
    pub fn CloseAudioDevice();
    pub fn IsAudioDeviceReady() -> bool;
    pub fn SetMasterVolume(arg_0: f32);
    pub fn GetMasterVolume() -> f32;
    pub fn LoadWave(arg_0: *const c_char) -> C_Wave;
    pub fn LoadWaveFromMemory(arg_0: *const c_char, arg_1: *mut u8, arg_2: c_int) -> C_Wave;
    pub fn IsWaveValid(arg_0: C_Wave) -> bool;
    pub fn LoadSound(arg_0: *const c_char) -> C_Sound;
    pub fn LoadSoundFromWave(arg_0: C_Wave) -> C_Sound;
    pub fn LoadSoundAlias(arg_0: C_Sound) -> C_Sound;
    pub fn IsSoundValid(arg_0: C_Sound) -> bool;
    pub fn UpdateSound(arg_0: C_Sound, arg_1: *mut c_void, arg_2: c_int);
    pub fn UnloadWave(arg_0: C_Wave);
    pub fn UnloadSound(arg_0: C_Sound);
    pub fn UnloadSoundAlias(arg_0: C_Sound);
    pub fn ExportWave(arg_0: C_Wave, arg_1: *const c_char) -> bool;
    pub fn ExportWaveAsCode(arg_0: C_Wave, arg_1: *const c_char) -> bool;
    pub fn PlaySound(arg_0: C_Sound);
    pub fn StopSound(arg_0: C_Sound);
    pub fn PauseSound(arg_0: C_Sound);
    pub fn ResumeSound(arg_0: C_Sound);
    pub fn IsSoundPlaying(arg_0: C_Sound) -> bool;
    pub fn SetSoundVolume(arg_0: C_Sound, arg_1: f32);
    pub fn SetSoundPitch(arg_0: C_Sound, arg_1: f32);
    pub fn SetSoundPan(arg_0: C_Sound, arg_1: f32);
    pub fn WaveCopy(arg_0: C_Wave) -> C_Wave;
    pub fn WaveCrop(arg_0: *mut C_Wave, arg_1: c_int, arg_2: c_int);
    pub fn WaveFormat(arg_0: *mut C_Wave, arg_1: c_int, arg_2: c_int, arg_3: c_int);
    pub fn LoadWaveSamples(arg_0: C_Wave) -> *mut f32;
    pub fn UnloadWaveSamples(arg_0: *mut f32);
    pub fn LoadMusicStream(arg_0: *const c_char) -> C_Music;
    pub fn LoadMusicStreamFromMemory(arg_0: *const c_char, arg_1: *mut u8, arg_2: c_int) -> C_Music;
    pub fn IsMusicValid(arg_0: C_Music) -> bool;
    pub fn UnloadMusicStream(arg_0: C_Music);
    pub fn PlayMusicStream(arg_0: C_Music);
    pub fn IsMusicStreamPlaying(arg_0: C_Music) -> bool;
    pub fn UpdateMusicStream(arg_0: C_Music);
    pub fn StopMusicStream(arg_0: C_Music);
    pub fn PauseMusicStream(arg_0: C_Music);
    pub fn ResumeMusicStream(arg_0: C_Music);
    pub fn SeekMusicStream(arg_0: C_Music, arg_1: f32);
    pub fn SetMusicVolume(arg_0: C_Music, arg_1: f32);
    pub fn SetMusicPitch(arg_0: C_Music, arg_1: f32);
    pub fn SetMusicPan(arg_0: C_Music, arg_1: f32);
    pub fn GetMusicTimeLength(arg_0: C_Music) -> f32;
    pub fn GetMusicTimePlayed(arg_0: C_Music) -> f32;
    pub fn LoadAudioStream(arg_0: c_uint, arg_1: c_uint, arg_2: c_uint) -> C_AudioStream;
    pub fn IsAudioStreamValid(arg_0: C_AudioStream) -> bool;
    pub fn UnloadAudioStream(arg_0: C_AudioStream);
    pub fn UpdateAudioStream(arg_0: C_AudioStream, arg_1: *mut c_void, arg_2: c_int);
    pub fn IsAudioStreamProcessed(arg_0: C_AudioStream) -> bool;
    pub fn PlayAudioStream(arg_0: C_AudioStream);
    pub fn PauseAudioStream(arg_0: C_AudioStream);
    pub fn ResumeAudioStream(arg_0: C_AudioStream);
    pub fn IsAudioStreamPlaying(arg_0: C_AudioStream) -> bool;
    pub fn StopAudioStream(arg_0: C_AudioStream);
    pub fn SetAudioStreamVolume(arg_0: C_AudioStream, arg_1: f32);
    pub fn SetAudioStreamPitch(arg_0: C_AudioStream, arg_1: f32);
    pub fn SetAudioStreamPan(arg_0: C_AudioStream, arg_1: f32);
    pub fn SetAudioStreamBufferSizeDefault(arg_0: c_int);
    pub fn SetAudioStreamCallback(arg_0: C_AudioStream, arg_1: AudioCallback);
    pub fn AttachAudioStreamProcessor(arg_0: C_AudioStream, arg_1: AudioCallback);
    pub fn DetachAudioStreamProcessor(arg_0: C_AudioStream, arg_1: AudioCallback);
    pub fn AttachAudioMixedProcessor(arg_0: AudioCallback);
    pub fn DetachAudioMixedProcessor(arg_0: AudioCallback);
}

// ===========================================================================
// Solix C-ABI Native Wrapper Functions
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_InitWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    InitWindow((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, str_ptr_2);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CloseWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    CloseWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_WindowShouldClose(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = WindowShouldClose();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowReady(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowReady();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowFullscreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowFullscreen();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowHidden(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowHidden();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowMinimized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowMinimized();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowMaximized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowMaximized();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowFocused(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowFocused();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowResized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowResized();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWindowState(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowState((*args.add(0)).payload as c_uint);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowState(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowState((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ClearWindowState(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ClearWindowState((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ToggleFullscreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ToggleFullscreen();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ToggleBorderlessWindowed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ToggleBorderlessWindowed();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MaximizeWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    MaximizeWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MinimizeWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    MinimizeWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_RestoreWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    RestoreWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowIcon(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowIcon(unpack_image(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowIcons(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    SetWindowIcons(&mut image_mut_0, (*args.add(1)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowTitle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    SetWindowTitle(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowPosition((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowMonitor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowMonitor((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowMinSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowMinSize((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowMaxSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowMaxSize((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowSize((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowOpacity(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowOpacity(f64::from_bits((*args.add(0)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetWindowFocused(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowFocused();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWindowHandle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWindowHandle();
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetScreenWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenWidth();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetScreenHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenHeight();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRenderWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRenderWidth();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRenderHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRenderHeight();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorCount();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCurrentMonitor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCurrentMonitor();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorPosition((*args.add(0)).payload as c_int);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorWidth((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorHeight((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorPhysicalWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorPhysicalWidth((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorPhysicalHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorPhysicalHeight((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorRefreshRate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorRefreshRate((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWindowPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWindowPosition();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWindowScaleDPI(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWindowScaleDPI();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMonitorName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorName((*args.add(0)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetClipboardText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    SetClipboardText(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetClipboardText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetClipboardText();
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetClipboardImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetClipboardImage();
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EnableEventWaiting(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EnableEventWaiting();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DisableEventWaiting(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DisableEventWaiting();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ShowCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ShowCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_HideCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    HideCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsCursorHidden(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsCursorHidden();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EnableCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EnableCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DisableCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DisableCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsCursorOnScreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsCursorOnScreen();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ClearBackground(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ClearBackground(unpack_color(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginDrawing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginDrawing();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndDrawing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndDrawing();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginMode2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginMode2D(unpack_camera2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndMode2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndMode2D();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginMode3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginMode3D(unpack_camera3d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndMode3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndMode3D();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginTextureMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginTextureMode(unpack_render_texture2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndTextureMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndTextureMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginShaderMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginShaderMode(unpack_shader(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndShaderMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndShaderMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginBlendMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginBlendMode((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndBlendMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndBlendMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginScissorMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginScissorMode((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndScissorMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndScissorMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_BeginVrStereoMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginVrStereoMode(unpack_vr_stereo_config(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EndVrStereoMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndVrStereoMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadVrStereoConfig(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadVrStereoConfig(unpack_vr_device_info(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_vr_stereo_config(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadVrStereoConfig(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadVrStereoConfig(unpack_vr_stereo_config(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadShader(str_ptr_0, str_ptr_1);
    SolixCValue::ref_val(pack_shader(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadShaderFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadShaderFromMemory(str_ptr_0, str_ptr_1);
    SolixCValue::ref_val(pack_shader(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsShaderValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsShaderValid(unpack_shader(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetShaderLocation(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetShaderLocation(unpack_shader(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetShaderLocationAttrib(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetShaderLocationAttrib(unpack_shader(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetShaderValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetShaderValue(unpack_shader(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_void, (*args.add(3)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetShaderValueV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetShaderValueV(unpack_shader(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_void, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetShaderValueMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetShaderValueMatrix(unpack_shader(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, unpack_matrix(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetShaderValueTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetShaderValueTexture(unpack_shader(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, unpack_texture2d(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadShader(unpack_shader(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetScreenToWorldRay(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenToWorldRay(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_camera(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_ray(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetScreenToWorldRayEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenToWorldRayEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_camera(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    SolixCValue::ref_val(pack_ray(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWorldToScreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWorldToScreen(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_camera(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWorldToScreenEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWorldToScreenEx(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_camera(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWorldToScreen2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWorldToScreen2D(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_camera2d(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetScreenToWorld2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenToWorld2D(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_camera2d(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCameraMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCameraMatrix(unpack_camera(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCameraMatrix2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCameraMatrix2D(unpack_camera2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetTargetFPS(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetTargetFPS((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFrameTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetFrameTime();
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTime();
    SolixCValue::f64_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFPS(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetFPS();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SwapScreenBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SwapScreenBuffer();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PollInputEvents(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PollInputEvents();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_WaitTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    WaitTime(f64::from_bits((*args.add(0)).payload));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetRandomSeed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetRandomSeed((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRandomValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRandomValue((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadRandomSequence(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadRandomSequence((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadRandomSequence(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadRandomSequence((*args.add(0)).payload as *mut c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TakeScreenshot(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    TakeScreenshot(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetConfigFlags(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetConfigFlags((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_OpenURL(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    OpenURL(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetTraceLogLevel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetTraceLogLevel((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TraceLog(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let log_level = (*args.add(0)).payload as c_int;
    let str_addr = (*args.add(1)).payload as u32;
    let mut str_buf = [0u8; 1024];
    let str_ptr = if str_addr > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr, str_buf.as_mut_ptr(), 1023) as usize;
        str_buf[read_len] = 0;
        str_buf.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    TraceLog(log_level, c"%s".as_ptr(), str_ptr);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetTraceLogCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetTraceLogCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MemAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = MemAlloc((*args.add(0)).payload as c_uint);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MemRealloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = MemRealloc((*args.add(0)).payload as *mut c_void, (*args.add(1)).payload as c_uint);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MemFree(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    MemFree((*args.add(0)).payload as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFileData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadFileData(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadFileData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadFileData((*args.add(0)).payload as *mut u8);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SaveFileData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = SaveFileData(str_ptr_0, (*args.add(1)).payload as *mut c_void, (*args.add(2)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportDataAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportDataAsCode((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int, str_ptr_2);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFileText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadFileText(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadFileText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    UnloadFileText(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SaveFileText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = SaveFileText(str_ptr_0, str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetLoadFileDataCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetLoadFileDataCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetSaveFileDataCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetSaveFileDataCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetLoadFileTextCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetLoadFileTextCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetSaveFileTextCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetSaveFileTextCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileRename(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileRename(str_ptr_0, str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileRemove(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileRemove(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileCopy(str_ptr_0, str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileMove(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileMove(str_ptr_0, str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileTextReplace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileTextReplace(str_ptr_0, str_ptr_1, str_ptr_2);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileTextFindIndex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileTextFindIndex(str_ptr_0, str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_FileExists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = FileExists(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DirectoryExists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = DirectoryExists(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsFileExtension(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = IsFileExtension(str_ptr_0, str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsFileHidden(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = IsFileHidden(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFileLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetFileLength(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFileModTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetFileModTime(str_ptr_0);
    SolixCValue::i64_val(call_result as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFileExtension(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetFileExtension(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFileName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetFileName(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFileNameWithoutExt(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetFileNameWithoutExt(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetDirectoryPath(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetDirectoryPath(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetPrevDirectoryPath(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetPrevDirectoryPath(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetWorkingDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWorkingDirectory();
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetApplicationDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetApplicationDirectory();
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MakeDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = MakeDirectory(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ChangeDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ChangeDirectory(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsPathFile(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = IsPathFile(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsPathDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = IsPathDirectory(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsPathAbsolute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = IsPathAbsolute(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsFileNameValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = IsFileNameValid(str_ptr_0);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadDirectoryFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadDirectoryFiles(str_ptr_0);
    SolixCValue::ref_val(pack_file_path_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadDirectoryFilesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadDirectoryFilesEx(str_ptr_0, str_ptr_1, (*args.add(2)).payload != 0);
    SolixCValue::ref_val(pack_file_path_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadDirectoryFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadDirectoryFiles(unpack_file_path_list(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsFileDropped(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsFileDropped();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadDroppedFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadDroppedFiles();
    SolixCValue::ref_val(pack_file_path_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadDroppedFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadDroppedFiles(unpack_file_path_list(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetDirectoryFileCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetDirectoryFileCount(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetDirectoryFileCountEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetDirectoryFileCountEx(str_ptr_0, str_ptr_1, (*args.add(2)).payload != 0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CompressData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CompressData((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DecompressData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = DecompressData((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_EncodeDataBase64(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = EncodeDataBase64((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DecodeDataBase64(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = DecodeDataBase64(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ComputeCRC32(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ComputeCRC32((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ComputeMD5(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ComputeMD5((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ComputeSHA1(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ComputeSHA1((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ComputeSHA256(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ComputeSHA256((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadAutomationEventList(str_ptr_0);
    SolixCValue::ref_val(pack_automation_event_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadAutomationEventList(unpack_automation_event_list(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportAutomationEventList(unpack_automation_event_list(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let automation_event_list_addr_0 = (*args.add(0)).payload as u32;
    let mut automation_event_list_mut_0 = unpack_automation_event_list(vm_context, automation_event_list_addr_0);
    SetAutomationEventList(&mut automation_event_list_mut_0);
    writeback_automation_event_list(vm_context, automation_event_list_addr_0, &automation_event_list_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAutomationEventBaseFrame(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetAutomationEventBaseFrame((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_StartAutomationEventRecording(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StartAutomationEventRecording();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_StopAutomationEventRecording(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StopAutomationEventRecording();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PlayAutomationEvent(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PlayAutomationEvent(unpack_automation_event(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsKeyPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsKeyPressed((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsKeyPressedRepeat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsKeyPressedRepeat((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsKeyDown(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsKeyDown((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsKeyReleased(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsKeyReleased((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsKeyUp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsKeyUp((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetKeyPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetKeyPressed();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCharPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCharPressed();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetKeyName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetKeyName((*args.add(0)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetExitKey(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetExitKey((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsGamepadAvailable(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsGamepadAvailable((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGamepadName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGamepadName((*args.add(0)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsGamepadButtonPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsGamepadButtonPressed((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsGamepadButtonDown(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsGamepadButtonDown((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsGamepadButtonReleased(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsGamepadButtonReleased((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsGamepadButtonUp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsGamepadButtonUp((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGamepadButtonPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGamepadButtonPressed();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGamepadAxisCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGamepadAxisCount((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGamepadAxisMovement(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGamepadAxisMovement((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetGamepadMappings(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = SetGamepadMappings(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetGamepadVibration(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetGamepadVibration((*args.add(0)).payload as c_int, f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMouseButtonPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMouseButtonPressed((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMouseButtonDown(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMouseButtonDown((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMouseButtonReleased(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMouseButtonReleased((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMouseButtonUp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMouseButtonUp((*args.add(0)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMouseX(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseX();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMouseY(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseY();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMousePosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMousePosition();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMouseDelta(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseDelta();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMousePosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMousePosition((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMouseOffset(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMouseOffset((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMouseScale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMouseScale(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMouseWheelMove(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseWheelMove();
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMouseWheelMoveV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseWheelMoveV();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMouseCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMouseCursor((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTouchX(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchX();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTouchY(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchY();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTouchPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchPosition((*args.add(0)).payload as c_int);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTouchPointId(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchPointId((*args.add(0)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTouchPointCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchPointCount();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetGesturesEnabled(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetGesturesEnabled((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsGestureDetected(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsGestureDetected((*args.add(0)).payload as c_uint);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGestureDetected(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureDetected();
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGestureHoldDuration(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureHoldDuration();
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGestureDragVector(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureDragVector();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGestureDragAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureDragAngle();
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGesturePinchVector(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGesturePinchVector();
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGesturePinchAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGesturePinchAngle();
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateCamera(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateCamera((*args.add(0)).payload as *mut C_Camera, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateCameraPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateCameraPro((*args.add(0)).payload as *mut C_Camera, unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetShapesTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetShapesTexture(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetShapesTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetShapesTexture();
    SolixCValue::ref_val(pack_texture2d(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetShapesTextureRectangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetShapesTextureRectangle();
    SolixCValue::ref_val(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPixel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPixel((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPixelV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPixelV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLine((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLineV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLineV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLineEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLineEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLineStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLineStrip((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLineBezier(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLineBezier(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLineDashed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLineDashed(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangle(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangleGradient(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangleLines(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangleLinesEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangleFan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangleFan((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangleStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangleStrip((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangle((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleRec(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectanglePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectanglePro(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleGradientV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleGradientV((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleGradientH(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleGradientH((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleGradientEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleGradientEx(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleLines((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleLinesEx(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleRounded(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleRounded(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleRoundedLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleRoundedLines(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRectangleRoundedLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRectangleRoundedLinesEx(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPoly(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPoly(unpack_vector2(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPolyLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPolyLines(unpack_vector2(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPolyLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPolyLinesEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircle((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleGradient(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_color(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleSector(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleSector(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleSectorLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleSectorLines(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleSectorLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleSectorLinesEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, f64::from_bits((*args.add(5)).payload) as f32, unpack_color(vm_context, (*args.add(6)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleLines((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleLinesV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleLinesV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircleLinesEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawEllipse(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawEllipse((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawEllipseV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawEllipseV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawEllipseLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawEllipseLines((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawEllipseLinesV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawEllipseLinesV(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawEllipseLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawEllipseLinesEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRing(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32, (*args.add(5)).payload as c_int, unpack_color(vm_context, (*args.add(6)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRingLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRingLines(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32, (*args.add(5)).payload as c_int, unpack_color(vm_context, (*args.add(6)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRingLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRingLinesEx(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32, (*args.add(5)).payload as c_int, f64::from_bits((*args.add(6)).payload) as f32, unpack_color(vm_context, (*args.add(7)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineLinear((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineBasis(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineBasis((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineCatmullRom(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineCatmullRom((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineBezierQuadratic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineBezierQuadratic((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineBezierCubic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineBezierCubic((*args.add(0)).payload as *const C_Vector2, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineSegmentLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineSegmentLinear(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineSegmentBasis(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineSegmentBasis(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineSegmentCatmullRom(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineSegmentCatmullRom(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineSegmentBezierQuadratic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineSegmentBezierQuadratic(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSplineSegmentBezierCubic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSplineSegmentBezierCubic(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetSplinePointLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetSplinePointLinear(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetSplinePointBasis(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetSplinePointBasis(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetSplinePointCatmullRom(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetSplinePointCatmullRom(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetSplinePointBezierQuadratic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetSplinePointBezierQuadratic(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetSplinePointBezierCubic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetSplinePointBezierCubic(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionRecs(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionRecs(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionCircles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionCircles(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionCircleRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionCircleRec(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_rectangle(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionCircleLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionCircleLine(unpack_vector2(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionPointRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionPointRec(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionPointCircle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionPointCircle(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionPointTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionPointTriangle(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionPointLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionPointLine(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), (*args.add(3)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionPointPoly(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionPointPoly(unpack_vector2(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as *const C_Vector2, (*args.add(2)).payload as c_int);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let vector2_addr_4 = (*args.add(4)).payload as u32;
    let mut vector2_mut_4 = unpack_vector2(vm_context, vector2_addr_4);
    let call_result = CheckCollisionLines(unpack_vector2(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), &mut vector2_mut_4);
    writeback_vector2(vm_context, vector2_addr_4, &vector2_mut_4);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCollisionRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCollisionRec(unpack_rectangle(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadImage(str_ptr_0);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageRaw(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadImageRaw(str_ptr_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageAnim(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadImageAnim(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageAnimFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadImageAnimFromMemory(str_ptr_0, (*args.add(1)).payload as *mut u8, (*args.add(2)).payload as c_int, (*args.add(3)).payload as *mut c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadImageFromMemory(str_ptr_0, (*args.add(1)).payload as *mut u8, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageFromTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadImageFromTexture(unpack_texture2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageFromScreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadImageFromScreen();
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsImageValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsImageValid(unpack_image(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadImage(unpack_image(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportImage(unpack_image(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportImageToMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportImageToMemory(unpack_image(vm_context, (*args.add(0)).payload as u32), str_ptr_1, (*args.add(2)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportImageAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportImageAsCode(unpack_image(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageColor((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageGradientLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageGradientLinear((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageGradientRadial(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageGradientRadial((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageGradientSquare(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageGradientSquare((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageChecked(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageChecked((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageWhiteNoise(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageWhiteNoise((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImagePerlinNoise(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImagePerlinNoise((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, f64::from_bits((*args.add(4)).payload) as f32);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageCellular(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageCellular((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GenImageText((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, str_ptr_2);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ImageCopy(unpack_image(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageFromImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ImageFromImage(unpack_image(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageFromChannel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ImageFromChannel(unpack_image(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ImageText(str_ptr_0, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ImageTextEx(unpack_font(vm_context, (*args.add(0)).payload as u32), str_ptr_1, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageFormat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageFormat(&mut image_mut_0, (*args.add(1)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageToPOT(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageToPOT(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageCrop(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageCrop(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageAlphaCrop(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaCrop(&mut image_mut_0, f64::from_bits((*args.add(1)).payload) as f32);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageAlphaClear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaClear(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageAlphaMask(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaMask(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageAlphaPremultiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaPremultiply(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageBlurGaussian(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageBlurGaussian(&mut image_mut_0, (*args.add(1)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageKernelConvolution(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageKernelConvolution(&mut image_mut_0, (*args.add(1)).payload as *mut f32, (*args.add(2)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageResize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageResize(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageResizeNN(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageResizeNN(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageResizeCanvas(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageResizeCanvas(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageMipmaps(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageMipmaps(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDither(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDither(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageFlipVertical(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageFlipVertical(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageFlipHorizontal(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageFlipHorizontal(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageRotate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageRotate(&mut image_mut_0, (*args.add(1)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageRotateCW(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageRotateCW(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageRotateCCW(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageRotateCCW(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageColorTint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorTint(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageColorInvert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorInvert(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageColorGrayscale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorGrayscale(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageColorContrast(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorContrast(&mut image_mut_0, (*args.add(1)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageColorBrightness(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorBrightness(&mut image_mut_0, (*args.add(1)).payload as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageColorReplace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorReplace(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImageColors(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadImageColors(unpack_image(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadImagePalette(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadImagePalette(unpack_image(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadImageColors(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let color_addr_0 = (*args.add(0)).payload as u32;
    let mut color_mut_0 = unpack_color(vm_context, color_addr_0);
    UnloadImageColors(&mut color_mut_0);
    writeback_color(vm_context, color_addr_0, &color_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadImagePalette(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let color_addr_0 = (*args.add(0)).payload as u32;
    let mut color_mut_0 = unpack_color(vm_context, color_addr_0);
    UnloadImagePalette(&mut color_mut_0);
    writeback_color(vm_context, color_addr_0, &color_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetImageAlphaBorder(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetImageAlphaBorder(unpack_image(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::ref_val(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetImageColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetImageColor(unpack_image(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageClearBackground(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageClearBackground(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawPixel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawPixel(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawPixelV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawPixelV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLine(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawLineV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLineV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawLineEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLineEx(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawLineStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLineStrip(&mut image_mut_0, (*args.add(1)).payload as *const C_Vector2, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangle(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTriangleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangleGradient(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32), unpack_color(vm_context, (*args.add(6)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTriangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangleLines(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTriangleFan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangleFan(&mut image_mut_0, (*args.add(1)).payload as *const C_Vector2, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTriangleStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangleStrip(&mut image_mut_0, (*args.add(1)).payload as *const C_Vector2, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangle(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectangleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectangleRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleRec(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectanglePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectanglePro(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleLines(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectangleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleLinesEx(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawRectangleGradientEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleGradientEx(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawCircle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircle(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawCircleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawCircleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleLines(&mut image_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawCircleLinesV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleLinesV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawCircleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleGradient(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImage(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawImageEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImageEx(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawImageRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImageRec(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).payload as u32), unpack_rectangle(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), unpack_color(vm_context, (*args.add(4)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawImagePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImagePro(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).payload as u32), unpack_rectangle(vm_context, (*args.add(2)).payload as u32), unpack_rectangle(vm_context, (*args.add(3)).payload as u32), unpack_vector2(vm_context, (*args.add(4)).payload as u32), f64::from_bits((*args.add(5)).payload) as f32, unpack_color(vm_context, (*args.add(6)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    ImageDrawText(&mut image_mut_0, str_ptr_1, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    ImageDrawTextEx(&mut image_mut_0, unpack_font(vm_context, (*args.add(1)).payload as u32), str_ptr_2, unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, f64::from_bits((*args.add(5)).payload) as f32, unpack_color(vm_context, (*args.add(6)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ImageDrawTextPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let image_addr_0 = (*args.add(0)).payload as u32;
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    ImageDrawTextPro(&mut image_mut_0, unpack_font(vm_context, (*args.add(1)).payload as u32), str_ptr_2, unpack_vector2(vm_context, (*args.add(3)).payload as u32), unpack_vector2(vm_context, (*args.add(4)).payload as u32), f64::from_bits((*args.add(5)).payload) as f32, f64::from_bits((*args.add(6)).payload) as f32, f64::from_bits((*args.add(7)).payload) as f32, unpack_color(vm_context, (*args.add(8)).payload as u32));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadTexture(str_ptr_0);
    SolixCValue::ref_val(pack_texture2d(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadTextureFromImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadTextureFromImage(unpack_image(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_texture2d(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadTextureCubemap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadTextureCubemap(unpack_image(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::ref_val(pack_texture_cubemap(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadRenderTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadRenderTexture((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::ref_val(pack_render_texture2d(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadRenderTextureEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadRenderTextureEx((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_render_texture2d(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsTextureValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsTextureValid(unpack_texture2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadTexture(unpack_texture2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsRenderTextureValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsRenderTextureValid(unpack_render_texture2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadRenderTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadRenderTexture(unpack_render_texture2d(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateTexture(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateTextureRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateTextureRec(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenTextureMipmaps(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    GenTextureMipmaps((*args.add(0)).payload as *mut C_Texture2D);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetTextureFilter(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetTextureFilter(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetTextureWrap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetTextureWrap(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTexture(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextureV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTextureV(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextureEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTextureEx(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextureRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTextureRec(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32), unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTexturePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTexturePro(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_rectangle(vm_context, (*args.add(1)).payload as u32), unpack_rectangle(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextureNPatch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTextureNPatch(unpack_texture2d(vm_context, (*args.add(0)).payload as u32), unpack_n_patch_info(vm_context, (*args.add(1)).payload as u32), unpack_rectangle(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorIsEqual(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorIsEqual(unpack_color(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_Fade(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Fade(unpack_color(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorToInt(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorToInt(unpack_color(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorNormalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorNormalize(unpack_color(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorFromNormalized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorFromNormalized(unpack_vector4(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorToHSV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorToHSV(unpack_color(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorFromHSV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorFromHSV(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorTint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorTint(unpack_color(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorBrightness(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorBrightness(unpack_color(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorContrast(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorContrast(unpack_color(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorAlpha(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorAlpha(unpack_color(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorAlphaBlend(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorAlphaBlend(unpack_color(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ColorLerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = ColorLerp(unpack_color(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetColor((*args.add(0)).payload as c_uint);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetPixelColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetPixelColor((*args.add(0)).payload as *mut c_void, (*args.add(1)).payload as c_int);
    SolixCValue::ref_val(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetPixelColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetPixelColor((*args.add(0)).payload as *mut c_void, unpack_color(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetPixelDataSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetPixelDataSize((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetFontDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetFontDefault();
    SolixCValue::ref_val(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFont(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadFont(str_ptr_0);
    SolixCValue::ref_val(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFontEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadFontEx(str_ptr_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_int, (*args.add(3)).payload as c_int);
    SolixCValue::ref_val(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFontFromImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadFontFromImage(unpack_image(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFontFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadFontFromMemory(str_ptr_0, (*args.add(1)).payload as *mut u8, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as *mut c_int, (*args.add(5)).payload as c_int);
    SolixCValue::ref_val(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsFontValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsFontValid(unpack_font(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadFontData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadFontData((*args.add(0)).payload as *mut u8, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as *mut c_int, (*args.add(4)).payload as c_int, (*args.add(5)).payload as c_int, (*args.add(6)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenImageFontAtlas(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenImageFontAtlas((*args.add(0)).payload as *const C_GlyphInfo, (*args.add(1)).payload as *mut *mut C_Rectangle, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, (*args.add(5)).payload as c_int);
    SolixCValue::ref_val(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadFontData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let glyph_info_addr_0 = (*args.add(0)).payload as u32;
    let mut glyph_info_mut_0 = unpack_glyph_info(vm_context, glyph_info_addr_0);
    UnloadFontData(&mut glyph_info_mut_0, (*args.add(1)).payload as c_int);
    writeback_glyph_info(vm_context, glyph_info_addr_0, &glyph_info_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadFont(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadFont(unpack_font(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportFontAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportFontAsCode(unpack_font(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawFPS(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawFPS((*args.add(0)).payload as c_int, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    DrawText(str_ptr_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    DrawTextEx(unpack_font(vm_context, (*args.add(0)).payload as u32), str_ptr_1, unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    DrawTextPro(unpack_font(vm_context, (*args.add(0)).payload as u32), str_ptr_1, unpack_vector2(vm_context, (*args.add(2)).payload as u32), unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, f64::from_bits((*args.add(5)).payload) as f32, f64::from_bits((*args.add(6)).payload) as f32, unpack_color(vm_context, (*args.add(7)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextCodepoint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTextCodepoint(unpack_font(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, unpack_vector2(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTextCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTextCodepoints(unpack_font(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as *mut c_int, (*args.add(2)).payload as c_int, unpack_vector2(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, f64::from_bits((*args.add(5)).payload) as f32, unpack_color(vm_context, (*args.add(6)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetTextLineSpacing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetTextLineSpacing((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MeasureText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = MeasureText(str_ptr_0, (*args.add(1)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MeasureTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = MeasureTextEx(unpack_font(vm_context, (*args.add(0)).payload as u32), str_ptr_1, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_MeasureTextCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = MeasureTextCodepoints(unpack_font(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as *mut c_int, (*args.add(2)).payload as c_int, f64::from_bits((*args.add(3)).payload) as f32, f64::from_bits((*args.add(4)).payload) as f32);
    SolixCValue::ref_val(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGlyphIndex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGlyphIndex(unpack_font(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGlyphInfo(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGlyphInfo(unpack_font(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::ref_val(pack_glyph_info(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetGlyphAtlasRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGlyphAtlasRec(unpack_font(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int);
    SolixCValue::ref_val(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadUTF8(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadUTF8((*args.add(0)).payload as *mut c_int, (*args.add(1)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadUTF8(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    UnloadUTF8(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadCodepoints(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadCodepoints((*args.add(0)).payload as *mut c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCodepointCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetCodepointCount(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCodepoint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetCodepoint(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCodepointNext(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetCodepointNext(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetCodepointPrevious(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetCodepointPrevious(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CodepointToUTF8(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CodepointToUTF8((*args.add(0)).payload as c_int, (*args.add(1)).payload as *mut c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadTextLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadTextLines(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadTextLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadTextLines((*args.add(0)).payload as *mut *mut c_char, (*args.add(1)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextCopy(str_ptr_0, str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextIsEqual(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextIsEqual(str_ptr_0, str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextLength(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextFormat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr = (*args.add(0)).payload as u32;
    let mut str_buf = [0u8; 1024];
    let str_ptr = if str_addr > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr, str_buf.as_mut_ptr(), 1023) as usize;
        str_buf[read_len] = 0;
        str_buf.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextFormat(c"%s".as_ptr(), str_ptr);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextSubtext(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextSubtext(str_ptr_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextRemoveSpaces(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextRemoveSpaces(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetTextBetween(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = GetTextBetween(str_ptr_0, str_ptr_1, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextReplace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextReplace(str_ptr_0, str_ptr_1, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextReplaceAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextReplaceAlloc(str_ptr_0, str_ptr_1, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextReplaceBetween(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_3 = (*args.add(3)).payload as u32;
    let mut str_buf_3 = [0u8; 1024];
    let str_ptr_3 = if str_addr_3 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_3, str_buf_3.as_mut_ptr(), 1023) as usize;
        str_buf_3[read_len] = 0;
        str_buf_3.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextReplaceBetween(str_ptr_0, str_ptr_1, str_ptr_2, str_ptr_3);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextReplaceBetweenAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_3 = (*args.add(3)).payload as u32;
    let mut str_buf_3 = [0u8; 1024];
    let str_ptr_3 = if str_addr_3 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_3, str_buf_3.as_mut_ptr(), 1023) as usize;
        str_buf_3[read_len] = 0;
        str_buf_3.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextReplaceBetweenAlloc(str_ptr_0, str_ptr_1, str_ptr_2, str_ptr_3);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextInsert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextInsert(str_ptr_0, str_ptr_1, (*args.add(2)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextInsertAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextInsertAlloc(str_ptr_0, str_ptr_1, (*args.add(2)).payload as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextJoin(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_2 = (*args.add(2)).payload as u32;
    let mut str_buf_2 = [0u8; 1024];
    let str_ptr_2 = if str_addr_2 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_2, str_buf_2.as_mut_ptr(), 1023) as usize;
        str_buf_2[read_len] = 0;
        str_buf_2.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextJoin((*args.add(0)).payload as *mut *mut c_char, (*args.add(1)).payload as c_int, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextSplit(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextSplit(str_ptr_0, (*args.add(1)).payload as c_char, (*args.add(2)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextAppend(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    TextAppend(str_ptr_0, str_ptr_1, (*args.add(2)).payload as *mut c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextFindIndex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextFindIndex(str_ptr_0, str_ptr_1);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToUpper(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToUpper(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToLower(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToLower(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToPascal(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToPascal(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToSnake(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToSnake(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToCamel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToCamel(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let c_str = CStr::from_ptr(call_result);
    let bytes = c_str.to_bytes();
    let context_ptr = vm_context as *mut SolixVmContext;
    let addr = ((*context_ptr).alloc_string)(vm_context, bytes.as_ptr(), bytes.len() as u32);
    SolixCValue::ref_val(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToInteger(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToInteger(str_ptr_0);
    SolixCValue::i32_val(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_TextToFloat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = TextToFloat(str_ptr_0);
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawLine3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawLine3D(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPoint3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPoint3D(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCircle3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCircle3D(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_vector3(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangle3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangle3D(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawTriangleStrip3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawTriangleStrip3D((*args.add(0)).payload as *const C_Vector3, (*args.add(1)).payload as c_int, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCube(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCube(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCubeV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCubeV(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCubeWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCubeWires(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCubeWiresV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCubeWiresV(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSphere(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSphereEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSphereEx(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawSphereWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawSphereWires(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCylinder(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCylinder(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCylinderEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCylinderEx(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCylinderWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCylinderWires(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCylinderWiresEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCylinderWiresEx(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, f64::from_bits((*args.add(3)).payload) as f32, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCapsule(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCapsule(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawCapsuleWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawCapsuleWires(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int, unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawPlane(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawPlane(unpack_vector3(vm_context, (*args.add(0)).payload as u32), unpack_vector2(vm_context, (*args.add(1)).payload as u32), unpack_color(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawRay(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawRay(unpack_ray(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawGrid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawGrid((*args.add(0)).payload as c_int, f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadModel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadModel(str_ptr_0);
    SolixCValue::ref_val(pack_model(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadModelFromMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadModelFromMesh(unpack_mesh(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_model(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsModelValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsModelValid(unpack_model(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadModel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadModel(unpack_model(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetModelBoundingBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetModelBoundingBox(unpack_model(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_bounding_box(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawModel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawModel(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawModelEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawModelEx(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_vector3(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawModelWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawModelWires(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_color(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawModelWiresEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawModelWiresEx(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_vector3(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawBoundingBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawBoundingBox(unpack_bounding_box(vm_context, (*args.add(0)).payload as u32), unpack_color(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawBillboard(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawBillboard(unpack_camera(vm_context, (*args.add(0)).payload as u32), unpack_texture2d(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32, unpack_color(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawBillboardRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawBillboardRec(unpack_camera(vm_context, (*args.add(0)).payload as u32), unpack_texture2d(vm_context, (*args.add(1)).payload as u32), unpack_rectangle(vm_context, (*args.add(2)).payload as u32), unpack_vector3(vm_context, (*args.add(3)).payload as u32), unpack_vector2(vm_context, (*args.add(4)).payload as u32), unpack_color(vm_context, (*args.add(5)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawBillboardPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawBillboardPro(unpack_camera(vm_context, (*args.add(0)).payload as u32), unpack_texture2d(vm_context, (*args.add(1)).payload as u32), unpack_rectangle(vm_context, (*args.add(2)).payload as u32), unpack_vector3(vm_context, (*args.add(3)).payload as u32), unpack_vector3(vm_context, (*args.add(4)).payload as u32), unpack_vector2(vm_context, (*args.add(5)).payload as u32), unpack_vector2(vm_context, (*args.add(6)).payload as u32), f64::from_bits((*args.add(7)).payload) as f32, unpack_color(vm_context, (*args.add(8)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UploadMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let mesh_addr_0 = (*args.add(0)).payload as u32;
    let mut mesh_mut_0 = unpack_mesh(vm_context, mesh_addr_0);
    UploadMesh(&mut mesh_mut_0, (*args.add(1)).payload != 0);
    writeback_mesh(vm_context, mesh_addr_0, &mesh_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateMeshBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateMeshBuffer(unpack_mesh(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as c_int, (*args.add(2)).payload as *mut c_void, (*args.add(3)).payload as c_int, (*args.add(4)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadMesh(unpack_mesh(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawMesh(unpack_mesh(vm_context, (*args.add(0)).payload as u32), unpack_material(vm_context, (*args.add(1)).payload as u32), unpack_matrix(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DrawMeshInstanced(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DrawMeshInstanced(unpack_mesh(vm_context, (*args.add(0)).payload as u32), unpack_material(vm_context, (*args.add(1)).payload as u32), (*args.add(2)).payload as *const C_Matrix, (*args.add(3)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMeshBoundingBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMeshBoundingBox(unpack_mesh(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_bounding_box(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshTangents(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let mesh_addr_0 = (*args.add(0)).payload as u32;
    let mut mesh_mut_0 = unpack_mesh(vm_context, mesh_addr_0);
    GenMeshTangents(&mut mesh_mut_0);
    writeback_mesh(vm_context, mesh_addr_0, &mesh_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportMesh(unpack_mesh(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportMeshAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportMeshAsCode(unpack_mesh(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshPoly(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshPoly((*args.add(0)).payload as c_int, f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshPlane(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshPlane(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshCube(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshCube(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshSphere(f64::from_bits((*args.add(0)).payload) as f32, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshHemiSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshHemiSphere(f64::from_bits((*args.add(0)).payload) as f32, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshCylinder(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshCylinder(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshCone(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshCone(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshTorus(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshTorus(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshKnot(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshKnot(f64::from_bits((*args.add(0)).payload) as f32, f64::from_bits((*args.add(1)).payload) as f32, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshHeightmap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshHeightmap(unpack_image(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GenMeshCubicmap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GenMeshCubicmap(unpack_image(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadMaterials(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadMaterials(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadMaterialDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadMaterialDefault();
    SolixCValue::ref_val(pack_material(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMaterialValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMaterialValid(unpack_material(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadMaterial(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadMaterial(unpack_material(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMaterialTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let material_addr_0 = (*args.add(0)).payload as u32;
    let mut material_mut_0 = unpack_material(vm_context, material_addr_0);
    SetMaterialTexture(&mut material_mut_0, (*args.add(1)).payload as c_int, unpack_texture2d(vm_context, (*args.add(2)).payload as u32));
    writeback_material(vm_context, material_addr_0, &material_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetModelMeshMaterial(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let model_addr_0 = (*args.add(0)).payload as u32;
    let mut model_mut_0 = unpack_model(vm_context, model_addr_0);
    SetModelMeshMaterial(&mut model_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    writeback_model(vm_context, model_addr_0, &model_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadModelAnimations(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadModelAnimations(str_ptr_0, (*args.add(1)).payload as *mut c_int);
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateModelAnimation(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateModelAnimation(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_model_animation(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateModelAnimationEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateModelAnimationEx(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_model_animation(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32, unpack_model_animation(vm_context, (*args.add(3)).payload as u32), f64::from_bits((*args.add(4)).payload) as f32, f64::from_bits((*args.add(5)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadModelAnimations(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let model_animation_addr_0 = (*args.add(0)).payload as u32;
    let mut model_animation_mut_0 = unpack_model_animation(vm_context, model_animation_addr_0);
    UnloadModelAnimations(&mut model_animation_mut_0, (*args.add(1)).payload as c_int);
    writeback_model_animation(vm_context, model_animation_addr_0, &model_animation_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsModelAnimationValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsModelAnimationValid(unpack_model(vm_context, (*args.add(0)).payload as u32), unpack_model_animation(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionSpheres(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionSpheres(unpack_vector3(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32, unpack_vector3(vm_context, (*args.add(2)).payload as u32), f64::from_bits((*args.add(3)).payload) as f32);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionBoxes(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionBoxes(unpack_bounding_box(vm_context, (*args.add(0)).payload as u32), unpack_bounding_box(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CheckCollisionBoxSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = CheckCollisionBoxSphere(unpack_bounding_box(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRayCollisionSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRayCollisionSphere(unpack_ray(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), f64::from_bits((*args.add(2)).payload) as f32);
    SolixCValue::ref_val(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRayCollisionBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRayCollisionBox(unpack_ray(vm_context, (*args.add(0)).payload as u32), unpack_bounding_box(vm_context, (*args.add(1)).payload as u32));
    SolixCValue::ref_val(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRayCollisionMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRayCollisionMesh(unpack_ray(vm_context, (*args.add(0)).payload as u32), unpack_mesh(vm_context, (*args.add(1)).payload as u32), unpack_matrix(vm_context, (*args.add(2)).payload as u32));
    SolixCValue::ref_val(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRayCollisionTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRayCollisionTriangle(unpack_ray(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), unpack_vector3(vm_context, (*args.add(3)).payload as u32));
    SolixCValue::ref_val(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetRayCollisionQuad(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRayCollisionQuad(unpack_ray(vm_context, (*args.add(0)).payload as u32), unpack_vector3(vm_context, (*args.add(1)).payload as u32), unpack_vector3(vm_context, (*args.add(2)).payload as u32), unpack_vector3(vm_context, (*args.add(3)).payload as u32), unpack_vector3(vm_context, (*args.add(4)).payload as u32));
    SolixCValue::ref_val(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_InitAudioDevice(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    InitAudioDevice();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_CloseAudioDevice(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    CloseAudioDevice();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsAudioDeviceReady(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsAudioDeviceReady();
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMasterVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMasterVolume(f64::from_bits((*args.add(0)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMasterVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMasterVolume();
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadWave(str_ptr_0);
    SolixCValue::ref_val(pack_wave(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadWaveFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadWaveFromMemory(str_ptr_0, (*args.add(1)).payload as *mut u8, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_wave(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsWaveValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWaveValid(unpack_wave(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadSound(str_ptr_0);
    SolixCValue::ref_val(pack_sound(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadSoundFromWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadSoundFromWave(unpack_wave(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_sound(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadSoundAlias(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadSoundAlias(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_sound(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsSoundValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsSoundValid(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateSound(unpack_sound(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as *mut c_void, (*args.add(2)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadWave(unpack_wave(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadSound(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadSoundAlias(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadSoundAlias(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportWave(unpack_wave(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ExportWaveAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_1 = (*args.add(1)).payload as u32;
    let mut str_buf_1 = [0u8; 1024];
    let str_ptr_1 = if str_addr_1 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_1, str_buf_1.as_mut_ptr(), 1023) as usize;
        str_buf_1[read_len] = 0;
        str_buf_1.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = ExportWaveAsCode(unpack_wave(vm_context, (*args.add(0)).payload as u32), str_ptr_1);
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PlaySound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PlaySound(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_StopSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StopSound(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PauseSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PauseSound(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ResumeSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ResumeSound(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsSoundPlaying(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsSoundPlaying(unpack_sound(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetSoundVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetSoundVolume(unpack_sound(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetSoundPitch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetSoundPitch(unpack_sound(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetSoundPan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetSoundPan(unpack_sound(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_WaveCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = WaveCopy(unpack_wave(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::ref_val(pack_wave(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_WaveCrop(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let wave_addr_0 = (*args.add(0)).payload as u32;
    let mut wave_mut_0 = unpack_wave(vm_context, wave_addr_0);
    WaveCrop(&mut wave_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int);
    writeback_wave(vm_context, wave_addr_0, &wave_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_WaveFormat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let wave_addr_0 = (*args.add(0)).payload as u32;
    let mut wave_mut_0 = unpack_wave(vm_context, wave_addr_0);
    WaveFormat(&mut wave_mut_0, (*args.add(1)).payload as c_int, (*args.add(2)).payload as c_int, (*args.add(3)).payload as c_int);
    writeback_wave(vm_context, wave_addr_0, &wave_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadWaveSamples(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadWaveSamples(unpack_wave(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::i64_val(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadWaveSamples(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadWaveSamples((*args.add(0)).payload as *mut f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadMusicStream(str_ptr_0);
    SolixCValue::ref_val(pack_music(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadMusicStreamFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let str_addr_0 = (*args.add(0)).payload as u32;
    let mut str_buf_0 = [0u8; 1024];
    let str_ptr_0 = if str_addr_0 > 0 {
        let context_ptr = vm_context as *mut SolixVmContext;
        let read_len = ((*context_ptr).read_bytes)(vm_context, str_addr_0, str_buf_0.as_mut_ptr(), 1023) as usize;
        str_buf_0[read_len] = 0;
        str_buf_0.as_ptr() as *const c_char
    } else {
        c"".as_ptr()
    };
    let call_result = LoadMusicStreamFromMemory(str_ptr_0, (*args.add(1)).payload as *mut u8, (*args.add(2)).payload as c_int);
    SolixCValue::ref_val(pack_music(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMusicValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMusicValid(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PlayMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PlayMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsMusicStreamPlaying(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsMusicStreamPlaying(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_StopMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StopMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PauseMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PauseMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ResumeMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ResumeMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SeekMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SeekMusicStream(unpack_music(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMusicVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMusicVolume(unpack_music(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMusicPitch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMusicPitch(unpack_music(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetMusicPan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetMusicPan(unpack_music(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMusicTimeLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMusicTimeLength(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_GetMusicTimePlayed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMusicTimePlayed(unpack_music(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::f64_val(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_LoadAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadAudioStream((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint, (*args.add(2)).payload as c_uint);
    SolixCValue::ref_val(pack_audio_stream(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsAudioStreamValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsAudioStreamValid(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UnloadAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UnloadAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_UpdateAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    UpdateAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), (*args.add(1)).payload as *mut c_void, (*args.add(2)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsAudioStreamProcessed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsAudioStreamProcessed(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PlayAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PlayAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_PauseAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PauseAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_ResumeAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ResumeAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_IsAudioStreamPlaying(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsAudioStreamPlaying(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::bool_val(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_StopAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StopAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAudioStreamVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetAudioStreamVolume(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAudioStreamPitch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetAudioStreamPitch(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAudioStreamPan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetAudioStreamPan(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), f64::from_bits((*args.add(1)).payload) as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAudioStreamBufferSizeDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetAudioStreamBufferSizeDefault((*args.add(0)).payload as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_SetAudioStreamCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetAudioStreamCallback(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), if (*args.add(1)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(1)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_AttachAudioStreamProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    AttachAudioStreamProcessor(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), if (*args.add(1)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(1)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DetachAudioStreamProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DetachAudioStreamProcessor(unpack_audio_stream(vm_context, (*args.add(0)).payload as u32), if (*args.add(1)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(1)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_AttachAudioMixedProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    AttachAudioMixedProcessor(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Raylib_DetachAudioMixedProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DetachAudioMixedProcessor(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

// ===========================================================================
// Native Symbol Dynamic Registration Hook
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut c_void,
    register_callback: RegisterNativeCallback,
) {
    register_callback(collector_context, c"Raylib::InitWindow".as_ptr(), Raylib_InitWindow);
    register_callback(collector_context, c"Raylib::init_window".as_ptr(), Raylib_InitWindow);
    register_callback(collector_context, c"raylib::InitWindow".as_ptr(), Raylib_InitWindow);
    register_callback(collector_context, c"raylib::init_window".as_ptr(), Raylib_InitWindow);
    register_callback(collector_context, c"InitWindow".as_ptr(), Raylib_InitWindow);
    register_callback(collector_context, c"init_window".as_ptr(), Raylib_InitWindow);
    register_callback(collector_context, c"Raylib::CloseWindow".as_ptr(), Raylib_CloseWindow);
    register_callback(collector_context, c"Raylib::close_window".as_ptr(), Raylib_CloseWindow);
    register_callback(collector_context, c"raylib::CloseWindow".as_ptr(), Raylib_CloseWindow);
    register_callback(collector_context, c"raylib::close_window".as_ptr(), Raylib_CloseWindow);
    register_callback(collector_context, c"CloseWindow".as_ptr(), Raylib_CloseWindow);
    register_callback(collector_context, c"close_window".as_ptr(), Raylib_CloseWindow);
    register_callback(collector_context, c"Raylib::WindowShouldClose".as_ptr(), Raylib_WindowShouldClose);
    register_callback(collector_context, c"Raylib::window_should_close".as_ptr(), Raylib_WindowShouldClose);
    register_callback(collector_context, c"raylib::WindowShouldClose".as_ptr(), Raylib_WindowShouldClose);
    register_callback(collector_context, c"raylib::window_should_close".as_ptr(), Raylib_WindowShouldClose);
    register_callback(collector_context, c"WindowShouldClose".as_ptr(), Raylib_WindowShouldClose);
    register_callback(collector_context, c"window_should_close".as_ptr(), Raylib_WindowShouldClose);
    register_callback(collector_context, c"Raylib::IsWindowReady".as_ptr(), Raylib_IsWindowReady);
    register_callback(collector_context, c"Raylib::is_window_ready".as_ptr(), Raylib_IsWindowReady);
    register_callback(collector_context, c"raylib::IsWindowReady".as_ptr(), Raylib_IsWindowReady);
    register_callback(collector_context, c"raylib::is_window_ready".as_ptr(), Raylib_IsWindowReady);
    register_callback(collector_context, c"IsWindowReady".as_ptr(), Raylib_IsWindowReady);
    register_callback(collector_context, c"is_window_ready".as_ptr(), Raylib_IsWindowReady);
    register_callback(collector_context, c"Raylib::IsWindowFullscreen".as_ptr(), Raylib_IsWindowFullscreen);
    register_callback(collector_context, c"Raylib::is_window_fullscreen".as_ptr(), Raylib_IsWindowFullscreen);
    register_callback(collector_context, c"raylib::IsWindowFullscreen".as_ptr(), Raylib_IsWindowFullscreen);
    register_callback(collector_context, c"raylib::is_window_fullscreen".as_ptr(), Raylib_IsWindowFullscreen);
    register_callback(collector_context, c"IsWindowFullscreen".as_ptr(), Raylib_IsWindowFullscreen);
    register_callback(collector_context, c"is_window_fullscreen".as_ptr(), Raylib_IsWindowFullscreen);
    register_callback(collector_context, c"Raylib::IsWindowHidden".as_ptr(), Raylib_IsWindowHidden);
    register_callback(collector_context, c"Raylib::is_window_hidden".as_ptr(), Raylib_IsWindowHidden);
    register_callback(collector_context, c"raylib::IsWindowHidden".as_ptr(), Raylib_IsWindowHidden);
    register_callback(collector_context, c"raylib::is_window_hidden".as_ptr(), Raylib_IsWindowHidden);
    register_callback(collector_context, c"IsWindowHidden".as_ptr(), Raylib_IsWindowHidden);
    register_callback(collector_context, c"is_window_hidden".as_ptr(), Raylib_IsWindowHidden);
    register_callback(collector_context, c"Raylib::IsWindowMinimized".as_ptr(), Raylib_IsWindowMinimized);
    register_callback(collector_context, c"Raylib::is_window_minimized".as_ptr(), Raylib_IsWindowMinimized);
    register_callback(collector_context, c"raylib::IsWindowMinimized".as_ptr(), Raylib_IsWindowMinimized);
    register_callback(collector_context, c"raylib::is_window_minimized".as_ptr(), Raylib_IsWindowMinimized);
    register_callback(collector_context, c"IsWindowMinimized".as_ptr(), Raylib_IsWindowMinimized);
    register_callback(collector_context, c"is_window_minimized".as_ptr(), Raylib_IsWindowMinimized);
    register_callback(collector_context, c"Raylib::IsWindowMaximized".as_ptr(), Raylib_IsWindowMaximized);
    register_callback(collector_context, c"Raylib::is_window_maximized".as_ptr(), Raylib_IsWindowMaximized);
    register_callback(collector_context, c"raylib::IsWindowMaximized".as_ptr(), Raylib_IsWindowMaximized);
    register_callback(collector_context, c"raylib::is_window_maximized".as_ptr(), Raylib_IsWindowMaximized);
    register_callback(collector_context, c"IsWindowMaximized".as_ptr(), Raylib_IsWindowMaximized);
    register_callback(collector_context, c"is_window_maximized".as_ptr(), Raylib_IsWindowMaximized);
    register_callback(collector_context, c"Raylib::IsWindowFocused".as_ptr(), Raylib_IsWindowFocused);
    register_callback(collector_context, c"Raylib::is_window_focused".as_ptr(), Raylib_IsWindowFocused);
    register_callback(collector_context, c"raylib::IsWindowFocused".as_ptr(), Raylib_IsWindowFocused);
    register_callback(collector_context, c"raylib::is_window_focused".as_ptr(), Raylib_IsWindowFocused);
    register_callback(collector_context, c"IsWindowFocused".as_ptr(), Raylib_IsWindowFocused);
    register_callback(collector_context, c"is_window_focused".as_ptr(), Raylib_IsWindowFocused);
    register_callback(collector_context, c"Raylib::IsWindowResized".as_ptr(), Raylib_IsWindowResized);
    register_callback(collector_context, c"Raylib::is_window_resized".as_ptr(), Raylib_IsWindowResized);
    register_callback(collector_context, c"raylib::IsWindowResized".as_ptr(), Raylib_IsWindowResized);
    register_callback(collector_context, c"raylib::is_window_resized".as_ptr(), Raylib_IsWindowResized);
    register_callback(collector_context, c"IsWindowResized".as_ptr(), Raylib_IsWindowResized);
    register_callback(collector_context, c"is_window_resized".as_ptr(), Raylib_IsWindowResized);
    register_callback(collector_context, c"Raylib::IsWindowState".as_ptr(), Raylib_IsWindowState);
    register_callback(collector_context, c"Raylib::is_window_state".as_ptr(), Raylib_IsWindowState);
    register_callback(collector_context, c"raylib::IsWindowState".as_ptr(), Raylib_IsWindowState);
    register_callback(collector_context, c"raylib::is_window_state".as_ptr(), Raylib_IsWindowState);
    register_callback(collector_context, c"IsWindowState".as_ptr(), Raylib_IsWindowState);
    register_callback(collector_context, c"is_window_state".as_ptr(), Raylib_IsWindowState);
    register_callback(collector_context, c"Raylib::SetWindowState".as_ptr(), Raylib_SetWindowState);
    register_callback(collector_context, c"Raylib::set_window_state".as_ptr(), Raylib_SetWindowState);
    register_callback(collector_context, c"raylib::SetWindowState".as_ptr(), Raylib_SetWindowState);
    register_callback(collector_context, c"raylib::set_window_state".as_ptr(), Raylib_SetWindowState);
    register_callback(collector_context, c"SetWindowState".as_ptr(), Raylib_SetWindowState);
    register_callback(collector_context, c"set_window_state".as_ptr(), Raylib_SetWindowState);
    register_callback(collector_context, c"Raylib::ClearWindowState".as_ptr(), Raylib_ClearWindowState);
    register_callback(collector_context, c"Raylib::clear_window_state".as_ptr(), Raylib_ClearWindowState);
    register_callback(collector_context, c"raylib::ClearWindowState".as_ptr(), Raylib_ClearWindowState);
    register_callback(collector_context, c"raylib::clear_window_state".as_ptr(), Raylib_ClearWindowState);
    register_callback(collector_context, c"ClearWindowState".as_ptr(), Raylib_ClearWindowState);
    register_callback(collector_context, c"clear_window_state".as_ptr(), Raylib_ClearWindowState);
    register_callback(collector_context, c"Raylib::ToggleFullscreen".as_ptr(), Raylib_ToggleFullscreen);
    register_callback(collector_context, c"Raylib::toggle_fullscreen".as_ptr(), Raylib_ToggleFullscreen);
    register_callback(collector_context, c"raylib::ToggleFullscreen".as_ptr(), Raylib_ToggleFullscreen);
    register_callback(collector_context, c"raylib::toggle_fullscreen".as_ptr(), Raylib_ToggleFullscreen);
    register_callback(collector_context, c"ToggleFullscreen".as_ptr(), Raylib_ToggleFullscreen);
    register_callback(collector_context, c"toggle_fullscreen".as_ptr(), Raylib_ToggleFullscreen);
    register_callback(collector_context, c"Raylib::ToggleBorderlessWindowed".as_ptr(), Raylib_ToggleBorderlessWindowed);
    register_callback(collector_context, c"Raylib::toggle_borderless_windowed".as_ptr(), Raylib_ToggleBorderlessWindowed);
    register_callback(collector_context, c"raylib::ToggleBorderlessWindowed".as_ptr(), Raylib_ToggleBorderlessWindowed);
    register_callback(collector_context, c"raylib::toggle_borderless_windowed".as_ptr(), Raylib_ToggleBorderlessWindowed);
    register_callback(collector_context, c"ToggleBorderlessWindowed".as_ptr(), Raylib_ToggleBorderlessWindowed);
    register_callback(collector_context, c"toggle_borderless_windowed".as_ptr(), Raylib_ToggleBorderlessWindowed);
    register_callback(collector_context, c"Raylib::MaximizeWindow".as_ptr(), Raylib_MaximizeWindow);
    register_callback(collector_context, c"Raylib::maximize_window".as_ptr(), Raylib_MaximizeWindow);
    register_callback(collector_context, c"raylib::MaximizeWindow".as_ptr(), Raylib_MaximizeWindow);
    register_callback(collector_context, c"raylib::maximize_window".as_ptr(), Raylib_MaximizeWindow);
    register_callback(collector_context, c"MaximizeWindow".as_ptr(), Raylib_MaximizeWindow);
    register_callback(collector_context, c"maximize_window".as_ptr(), Raylib_MaximizeWindow);
    register_callback(collector_context, c"Raylib::MinimizeWindow".as_ptr(), Raylib_MinimizeWindow);
    register_callback(collector_context, c"Raylib::minimize_window".as_ptr(), Raylib_MinimizeWindow);
    register_callback(collector_context, c"raylib::MinimizeWindow".as_ptr(), Raylib_MinimizeWindow);
    register_callback(collector_context, c"raylib::minimize_window".as_ptr(), Raylib_MinimizeWindow);
    register_callback(collector_context, c"MinimizeWindow".as_ptr(), Raylib_MinimizeWindow);
    register_callback(collector_context, c"minimize_window".as_ptr(), Raylib_MinimizeWindow);
    register_callback(collector_context, c"Raylib::RestoreWindow".as_ptr(), Raylib_RestoreWindow);
    register_callback(collector_context, c"Raylib::restore_window".as_ptr(), Raylib_RestoreWindow);
    register_callback(collector_context, c"raylib::RestoreWindow".as_ptr(), Raylib_RestoreWindow);
    register_callback(collector_context, c"raylib::restore_window".as_ptr(), Raylib_RestoreWindow);
    register_callback(collector_context, c"RestoreWindow".as_ptr(), Raylib_RestoreWindow);
    register_callback(collector_context, c"restore_window".as_ptr(), Raylib_RestoreWindow);
    register_callback(collector_context, c"Raylib::SetWindowIcon".as_ptr(), Raylib_SetWindowIcon);
    register_callback(collector_context, c"Raylib::set_window_icon".as_ptr(), Raylib_SetWindowIcon);
    register_callback(collector_context, c"raylib::SetWindowIcon".as_ptr(), Raylib_SetWindowIcon);
    register_callback(collector_context, c"raylib::set_window_icon".as_ptr(), Raylib_SetWindowIcon);
    register_callback(collector_context, c"SetWindowIcon".as_ptr(), Raylib_SetWindowIcon);
    register_callback(collector_context, c"set_window_icon".as_ptr(), Raylib_SetWindowIcon);
    register_callback(collector_context, c"Raylib::SetWindowIcons".as_ptr(), Raylib_SetWindowIcons);
    register_callback(collector_context, c"Raylib::set_window_icons".as_ptr(), Raylib_SetWindowIcons);
    register_callback(collector_context, c"raylib::SetWindowIcons".as_ptr(), Raylib_SetWindowIcons);
    register_callback(collector_context, c"raylib::set_window_icons".as_ptr(), Raylib_SetWindowIcons);
    register_callback(collector_context, c"SetWindowIcons".as_ptr(), Raylib_SetWindowIcons);
    register_callback(collector_context, c"set_window_icons".as_ptr(), Raylib_SetWindowIcons);
    register_callback(collector_context, c"Raylib::SetWindowTitle".as_ptr(), Raylib_SetWindowTitle);
    register_callback(collector_context, c"Raylib::set_window_title".as_ptr(), Raylib_SetWindowTitle);
    register_callback(collector_context, c"raylib::SetWindowTitle".as_ptr(), Raylib_SetWindowTitle);
    register_callback(collector_context, c"raylib::set_window_title".as_ptr(), Raylib_SetWindowTitle);
    register_callback(collector_context, c"SetWindowTitle".as_ptr(), Raylib_SetWindowTitle);
    register_callback(collector_context, c"set_window_title".as_ptr(), Raylib_SetWindowTitle);
    register_callback(collector_context, c"Raylib::SetWindowPosition".as_ptr(), Raylib_SetWindowPosition);
    register_callback(collector_context, c"Raylib::set_window_position".as_ptr(), Raylib_SetWindowPosition);
    register_callback(collector_context, c"raylib::SetWindowPosition".as_ptr(), Raylib_SetWindowPosition);
    register_callback(collector_context, c"raylib::set_window_position".as_ptr(), Raylib_SetWindowPosition);
    register_callback(collector_context, c"SetWindowPosition".as_ptr(), Raylib_SetWindowPosition);
    register_callback(collector_context, c"set_window_position".as_ptr(), Raylib_SetWindowPosition);
    register_callback(collector_context, c"Raylib::SetWindowMonitor".as_ptr(), Raylib_SetWindowMonitor);
    register_callback(collector_context, c"Raylib::set_window_monitor".as_ptr(), Raylib_SetWindowMonitor);
    register_callback(collector_context, c"raylib::SetWindowMonitor".as_ptr(), Raylib_SetWindowMonitor);
    register_callback(collector_context, c"raylib::set_window_monitor".as_ptr(), Raylib_SetWindowMonitor);
    register_callback(collector_context, c"SetWindowMonitor".as_ptr(), Raylib_SetWindowMonitor);
    register_callback(collector_context, c"set_window_monitor".as_ptr(), Raylib_SetWindowMonitor);
    register_callback(collector_context, c"Raylib::SetWindowMinSize".as_ptr(), Raylib_SetWindowMinSize);
    register_callback(collector_context, c"Raylib::set_window_min_size".as_ptr(), Raylib_SetWindowMinSize);
    register_callback(collector_context, c"raylib::SetWindowMinSize".as_ptr(), Raylib_SetWindowMinSize);
    register_callback(collector_context, c"raylib::set_window_min_size".as_ptr(), Raylib_SetWindowMinSize);
    register_callback(collector_context, c"SetWindowMinSize".as_ptr(), Raylib_SetWindowMinSize);
    register_callback(collector_context, c"set_window_min_size".as_ptr(), Raylib_SetWindowMinSize);
    register_callback(collector_context, c"Raylib::SetWindowMaxSize".as_ptr(), Raylib_SetWindowMaxSize);
    register_callback(collector_context, c"Raylib::set_window_max_size".as_ptr(), Raylib_SetWindowMaxSize);
    register_callback(collector_context, c"raylib::SetWindowMaxSize".as_ptr(), Raylib_SetWindowMaxSize);
    register_callback(collector_context, c"raylib::set_window_max_size".as_ptr(), Raylib_SetWindowMaxSize);
    register_callback(collector_context, c"SetWindowMaxSize".as_ptr(), Raylib_SetWindowMaxSize);
    register_callback(collector_context, c"set_window_max_size".as_ptr(), Raylib_SetWindowMaxSize);
    register_callback(collector_context, c"Raylib::SetWindowSize".as_ptr(), Raylib_SetWindowSize);
    register_callback(collector_context, c"Raylib::set_window_size".as_ptr(), Raylib_SetWindowSize);
    register_callback(collector_context, c"raylib::SetWindowSize".as_ptr(), Raylib_SetWindowSize);
    register_callback(collector_context, c"raylib::set_window_size".as_ptr(), Raylib_SetWindowSize);
    register_callback(collector_context, c"SetWindowSize".as_ptr(), Raylib_SetWindowSize);
    register_callback(collector_context, c"set_window_size".as_ptr(), Raylib_SetWindowSize);
    register_callback(collector_context, c"Raylib::SetWindowOpacity".as_ptr(), Raylib_SetWindowOpacity);
    register_callback(collector_context, c"Raylib::set_window_opacity".as_ptr(), Raylib_SetWindowOpacity);
    register_callback(collector_context, c"raylib::SetWindowOpacity".as_ptr(), Raylib_SetWindowOpacity);
    register_callback(collector_context, c"raylib::set_window_opacity".as_ptr(), Raylib_SetWindowOpacity);
    register_callback(collector_context, c"SetWindowOpacity".as_ptr(), Raylib_SetWindowOpacity);
    register_callback(collector_context, c"set_window_opacity".as_ptr(), Raylib_SetWindowOpacity);
    register_callback(collector_context, c"Raylib::SetWindowFocused".as_ptr(), Raylib_SetWindowFocused);
    register_callback(collector_context, c"Raylib::set_window_focused".as_ptr(), Raylib_SetWindowFocused);
    register_callback(collector_context, c"raylib::SetWindowFocused".as_ptr(), Raylib_SetWindowFocused);
    register_callback(collector_context, c"raylib::set_window_focused".as_ptr(), Raylib_SetWindowFocused);
    register_callback(collector_context, c"SetWindowFocused".as_ptr(), Raylib_SetWindowFocused);
    register_callback(collector_context, c"set_window_focused".as_ptr(), Raylib_SetWindowFocused);
    register_callback(collector_context, c"Raylib::GetWindowHandle".as_ptr(), Raylib_GetWindowHandle);
    register_callback(collector_context, c"Raylib::get_window_handle".as_ptr(), Raylib_GetWindowHandle);
    register_callback(collector_context, c"raylib::GetWindowHandle".as_ptr(), Raylib_GetWindowHandle);
    register_callback(collector_context, c"raylib::get_window_handle".as_ptr(), Raylib_GetWindowHandle);
    register_callback(collector_context, c"GetWindowHandle".as_ptr(), Raylib_GetWindowHandle);
    register_callback(collector_context, c"get_window_handle".as_ptr(), Raylib_GetWindowHandle);
    register_callback(collector_context, c"Raylib::GetScreenWidth".as_ptr(), Raylib_GetScreenWidth);
    register_callback(collector_context, c"Raylib::get_screen_width".as_ptr(), Raylib_GetScreenWidth);
    register_callback(collector_context, c"raylib::GetScreenWidth".as_ptr(), Raylib_GetScreenWidth);
    register_callback(collector_context, c"raylib::get_screen_width".as_ptr(), Raylib_GetScreenWidth);
    register_callback(collector_context, c"GetScreenWidth".as_ptr(), Raylib_GetScreenWidth);
    register_callback(collector_context, c"get_screen_width".as_ptr(), Raylib_GetScreenWidth);
    register_callback(collector_context, c"Raylib::GetScreenHeight".as_ptr(), Raylib_GetScreenHeight);
    register_callback(collector_context, c"Raylib::get_screen_height".as_ptr(), Raylib_GetScreenHeight);
    register_callback(collector_context, c"raylib::GetScreenHeight".as_ptr(), Raylib_GetScreenHeight);
    register_callback(collector_context, c"raylib::get_screen_height".as_ptr(), Raylib_GetScreenHeight);
    register_callback(collector_context, c"GetScreenHeight".as_ptr(), Raylib_GetScreenHeight);
    register_callback(collector_context, c"get_screen_height".as_ptr(), Raylib_GetScreenHeight);
    register_callback(collector_context, c"Raylib::GetRenderWidth".as_ptr(), Raylib_GetRenderWidth);
    register_callback(collector_context, c"Raylib::get_render_width".as_ptr(), Raylib_GetRenderWidth);
    register_callback(collector_context, c"raylib::GetRenderWidth".as_ptr(), Raylib_GetRenderWidth);
    register_callback(collector_context, c"raylib::get_render_width".as_ptr(), Raylib_GetRenderWidth);
    register_callback(collector_context, c"GetRenderWidth".as_ptr(), Raylib_GetRenderWidth);
    register_callback(collector_context, c"get_render_width".as_ptr(), Raylib_GetRenderWidth);
    register_callback(collector_context, c"Raylib::GetRenderHeight".as_ptr(), Raylib_GetRenderHeight);
    register_callback(collector_context, c"Raylib::get_render_height".as_ptr(), Raylib_GetRenderHeight);
    register_callback(collector_context, c"raylib::GetRenderHeight".as_ptr(), Raylib_GetRenderHeight);
    register_callback(collector_context, c"raylib::get_render_height".as_ptr(), Raylib_GetRenderHeight);
    register_callback(collector_context, c"GetRenderHeight".as_ptr(), Raylib_GetRenderHeight);
    register_callback(collector_context, c"get_render_height".as_ptr(), Raylib_GetRenderHeight);
    register_callback(collector_context, c"Raylib::GetMonitorCount".as_ptr(), Raylib_GetMonitorCount);
    register_callback(collector_context, c"Raylib::get_monitor_count".as_ptr(), Raylib_GetMonitorCount);
    register_callback(collector_context, c"raylib::GetMonitorCount".as_ptr(), Raylib_GetMonitorCount);
    register_callback(collector_context, c"raylib::get_monitor_count".as_ptr(), Raylib_GetMonitorCount);
    register_callback(collector_context, c"GetMonitorCount".as_ptr(), Raylib_GetMonitorCount);
    register_callback(collector_context, c"get_monitor_count".as_ptr(), Raylib_GetMonitorCount);
    register_callback(collector_context, c"Raylib::GetCurrentMonitor".as_ptr(), Raylib_GetCurrentMonitor);
    register_callback(collector_context, c"Raylib::get_current_monitor".as_ptr(), Raylib_GetCurrentMonitor);
    register_callback(collector_context, c"raylib::GetCurrentMonitor".as_ptr(), Raylib_GetCurrentMonitor);
    register_callback(collector_context, c"raylib::get_current_monitor".as_ptr(), Raylib_GetCurrentMonitor);
    register_callback(collector_context, c"GetCurrentMonitor".as_ptr(), Raylib_GetCurrentMonitor);
    register_callback(collector_context, c"get_current_monitor".as_ptr(), Raylib_GetCurrentMonitor);
    register_callback(collector_context, c"Raylib::GetMonitorPosition".as_ptr(), Raylib_GetMonitorPosition);
    register_callback(collector_context, c"Raylib::get_monitor_position".as_ptr(), Raylib_GetMonitorPosition);
    register_callback(collector_context, c"raylib::GetMonitorPosition".as_ptr(), Raylib_GetMonitorPosition);
    register_callback(collector_context, c"raylib::get_monitor_position".as_ptr(), Raylib_GetMonitorPosition);
    register_callback(collector_context, c"GetMonitorPosition".as_ptr(), Raylib_GetMonitorPosition);
    register_callback(collector_context, c"get_monitor_position".as_ptr(), Raylib_GetMonitorPosition);
    register_callback(collector_context, c"Raylib::GetMonitorWidth".as_ptr(), Raylib_GetMonitorWidth);
    register_callback(collector_context, c"Raylib::get_monitor_width".as_ptr(), Raylib_GetMonitorWidth);
    register_callback(collector_context, c"raylib::GetMonitorWidth".as_ptr(), Raylib_GetMonitorWidth);
    register_callback(collector_context, c"raylib::get_monitor_width".as_ptr(), Raylib_GetMonitorWidth);
    register_callback(collector_context, c"GetMonitorWidth".as_ptr(), Raylib_GetMonitorWidth);
    register_callback(collector_context, c"get_monitor_width".as_ptr(), Raylib_GetMonitorWidth);
    register_callback(collector_context, c"Raylib::GetMonitorHeight".as_ptr(), Raylib_GetMonitorHeight);
    register_callback(collector_context, c"Raylib::get_monitor_height".as_ptr(), Raylib_GetMonitorHeight);
    register_callback(collector_context, c"raylib::GetMonitorHeight".as_ptr(), Raylib_GetMonitorHeight);
    register_callback(collector_context, c"raylib::get_monitor_height".as_ptr(), Raylib_GetMonitorHeight);
    register_callback(collector_context, c"GetMonitorHeight".as_ptr(), Raylib_GetMonitorHeight);
    register_callback(collector_context, c"get_monitor_height".as_ptr(), Raylib_GetMonitorHeight);
    register_callback(collector_context, c"Raylib::GetMonitorPhysicalWidth".as_ptr(), Raylib_GetMonitorPhysicalWidth);
    register_callback(collector_context, c"Raylib::get_monitor_physical_width".as_ptr(), Raylib_GetMonitorPhysicalWidth);
    register_callback(collector_context, c"raylib::GetMonitorPhysicalWidth".as_ptr(), Raylib_GetMonitorPhysicalWidth);
    register_callback(collector_context, c"raylib::get_monitor_physical_width".as_ptr(), Raylib_GetMonitorPhysicalWidth);
    register_callback(collector_context, c"GetMonitorPhysicalWidth".as_ptr(), Raylib_GetMonitorPhysicalWidth);
    register_callback(collector_context, c"get_monitor_physical_width".as_ptr(), Raylib_GetMonitorPhysicalWidth);
    register_callback(collector_context, c"Raylib::GetMonitorPhysicalHeight".as_ptr(), Raylib_GetMonitorPhysicalHeight);
    register_callback(collector_context, c"Raylib::get_monitor_physical_height".as_ptr(), Raylib_GetMonitorPhysicalHeight);
    register_callback(collector_context, c"raylib::GetMonitorPhysicalHeight".as_ptr(), Raylib_GetMonitorPhysicalHeight);
    register_callback(collector_context, c"raylib::get_monitor_physical_height".as_ptr(), Raylib_GetMonitorPhysicalHeight);
    register_callback(collector_context, c"GetMonitorPhysicalHeight".as_ptr(), Raylib_GetMonitorPhysicalHeight);
    register_callback(collector_context, c"get_monitor_physical_height".as_ptr(), Raylib_GetMonitorPhysicalHeight);
    register_callback(collector_context, c"Raylib::GetMonitorRefreshRate".as_ptr(), Raylib_GetMonitorRefreshRate);
    register_callback(collector_context, c"Raylib::get_monitor_refresh_rate".as_ptr(), Raylib_GetMonitorRefreshRate);
    register_callback(collector_context, c"raylib::GetMonitorRefreshRate".as_ptr(), Raylib_GetMonitorRefreshRate);
    register_callback(collector_context, c"raylib::get_monitor_refresh_rate".as_ptr(), Raylib_GetMonitorRefreshRate);
    register_callback(collector_context, c"GetMonitorRefreshRate".as_ptr(), Raylib_GetMonitorRefreshRate);
    register_callback(collector_context, c"get_monitor_refresh_rate".as_ptr(), Raylib_GetMonitorRefreshRate);
    register_callback(collector_context, c"Raylib::GetWindowPosition".as_ptr(), Raylib_GetWindowPosition);
    register_callback(collector_context, c"Raylib::get_window_position".as_ptr(), Raylib_GetWindowPosition);
    register_callback(collector_context, c"raylib::GetWindowPosition".as_ptr(), Raylib_GetWindowPosition);
    register_callback(collector_context, c"raylib::get_window_position".as_ptr(), Raylib_GetWindowPosition);
    register_callback(collector_context, c"GetWindowPosition".as_ptr(), Raylib_GetWindowPosition);
    register_callback(collector_context, c"get_window_position".as_ptr(), Raylib_GetWindowPosition);
    register_callback(collector_context, c"Raylib::GetWindowScaleDPI".as_ptr(), Raylib_GetWindowScaleDPI);
    register_callback(collector_context, c"Raylib::get_window_scale_dpi".as_ptr(), Raylib_GetWindowScaleDPI);
    register_callback(collector_context, c"raylib::GetWindowScaleDPI".as_ptr(), Raylib_GetWindowScaleDPI);
    register_callback(collector_context, c"raylib::get_window_scale_dpi".as_ptr(), Raylib_GetWindowScaleDPI);
    register_callback(collector_context, c"GetWindowScaleDPI".as_ptr(), Raylib_GetWindowScaleDPI);
    register_callback(collector_context, c"get_window_scale_dpi".as_ptr(), Raylib_GetWindowScaleDPI);
    register_callback(collector_context, c"Raylib::GetMonitorName".as_ptr(), Raylib_GetMonitorName);
    register_callback(collector_context, c"Raylib::get_monitor_name".as_ptr(), Raylib_GetMonitorName);
    register_callback(collector_context, c"raylib::GetMonitorName".as_ptr(), Raylib_GetMonitorName);
    register_callback(collector_context, c"raylib::get_monitor_name".as_ptr(), Raylib_GetMonitorName);
    register_callback(collector_context, c"GetMonitorName".as_ptr(), Raylib_GetMonitorName);
    register_callback(collector_context, c"get_monitor_name".as_ptr(), Raylib_GetMonitorName);
    register_callback(collector_context, c"Raylib::SetClipboardText".as_ptr(), Raylib_SetClipboardText);
    register_callback(collector_context, c"Raylib::set_clipboard_text".as_ptr(), Raylib_SetClipboardText);
    register_callback(collector_context, c"raylib::SetClipboardText".as_ptr(), Raylib_SetClipboardText);
    register_callback(collector_context, c"raylib::set_clipboard_text".as_ptr(), Raylib_SetClipboardText);
    register_callback(collector_context, c"SetClipboardText".as_ptr(), Raylib_SetClipboardText);
    register_callback(collector_context, c"set_clipboard_text".as_ptr(), Raylib_SetClipboardText);
    register_callback(collector_context, c"Raylib::GetClipboardText".as_ptr(), Raylib_GetClipboardText);
    register_callback(collector_context, c"Raylib::get_clipboard_text".as_ptr(), Raylib_GetClipboardText);
    register_callback(collector_context, c"raylib::GetClipboardText".as_ptr(), Raylib_GetClipboardText);
    register_callback(collector_context, c"raylib::get_clipboard_text".as_ptr(), Raylib_GetClipboardText);
    register_callback(collector_context, c"GetClipboardText".as_ptr(), Raylib_GetClipboardText);
    register_callback(collector_context, c"get_clipboard_text".as_ptr(), Raylib_GetClipboardText);
    register_callback(collector_context, c"Raylib::GetClipboardImage".as_ptr(), Raylib_GetClipboardImage);
    register_callback(collector_context, c"Raylib::get_clipboard_image".as_ptr(), Raylib_GetClipboardImage);
    register_callback(collector_context, c"raylib::GetClipboardImage".as_ptr(), Raylib_GetClipboardImage);
    register_callback(collector_context, c"raylib::get_clipboard_image".as_ptr(), Raylib_GetClipboardImage);
    register_callback(collector_context, c"GetClipboardImage".as_ptr(), Raylib_GetClipboardImage);
    register_callback(collector_context, c"get_clipboard_image".as_ptr(), Raylib_GetClipboardImage);
    register_callback(collector_context, c"Raylib::EnableEventWaiting".as_ptr(), Raylib_EnableEventWaiting);
    register_callback(collector_context, c"Raylib::enable_event_waiting".as_ptr(), Raylib_EnableEventWaiting);
    register_callback(collector_context, c"raylib::EnableEventWaiting".as_ptr(), Raylib_EnableEventWaiting);
    register_callback(collector_context, c"raylib::enable_event_waiting".as_ptr(), Raylib_EnableEventWaiting);
    register_callback(collector_context, c"EnableEventWaiting".as_ptr(), Raylib_EnableEventWaiting);
    register_callback(collector_context, c"enable_event_waiting".as_ptr(), Raylib_EnableEventWaiting);
    register_callback(collector_context, c"Raylib::DisableEventWaiting".as_ptr(), Raylib_DisableEventWaiting);
    register_callback(collector_context, c"Raylib::disable_event_waiting".as_ptr(), Raylib_DisableEventWaiting);
    register_callback(collector_context, c"raylib::DisableEventWaiting".as_ptr(), Raylib_DisableEventWaiting);
    register_callback(collector_context, c"raylib::disable_event_waiting".as_ptr(), Raylib_DisableEventWaiting);
    register_callback(collector_context, c"DisableEventWaiting".as_ptr(), Raylib_DisableEventWaiting);
    register_callback(collector_context, c"disable_event_waiting".as_ptr(), Raylib_DisableEventWaiting);
    register_callback(collector_context, c"Raylib::ShowCursor".as_ptr(), Raylib_ShowCursor);
    register_callback(collector_context, c"Raylib::show_cursor".as_ptr(), Raylib_ShowCursor);
    register_callback(collector_context, c"raylib::ShowCursor".as_ptr(), Raylib_ShowCursor);
    register_callback(collector_context, c"raylib::show_cursor".as_ptr(), Raylib_ShowCursor);
    register_callback(collector_context, c"ShowCursor".as_ptr(), Raylib_ShowCursor);
    register_callback(collector_context, c"show_cursor".as_ptr(), Raylib_ShowCursor);
    register_callback(collector_context, c"Raylib::HideCursor".as_ptr(), Raylib_HideCursor);
    register_callback(collector_context, c"Raylib::hide_cursor".as_ptr(), Raylib_HideCursor);
    register_callback(collector_context, c"raylib::HideCursor".as_ptr(), Raylib_HideCursor);
    register_callback(collector_context, c"raylib::hide_cursor".as_ptr(), Raylib_HideCursor);
    register_callback(collector_context, c"HideCursor".as_ptr(), Raylib_HideCursor);
    register_callback(collector_context, c"hide_cursor".as_ptr(), Raylib_HideCursor);
    register_callback(collector_context, c"Raylib::IsCursorHidden".as_ptr(), Raylib_IsCursorHidden);
    register_callback(collector_context, c"Raylib::is_cursor_hidden".as_ptr(), Raylib_IsCursorHidden);
    register_callback(collector_context, c"raylib::IsCursorHidden".as_ptr(), Raylib_IsCursorHidden);
    register_callback(collector_context, c"raylib::is_cursor_hidden".as_ptr(), Raylib_IsCursorHidden);
    register_callback(collector_context, c"IsCursorHidden".as_ptr(), Raylib_IsCursorHidden);
    register_callback(collector_context, c"is_cursor_hidden".as_ptr(), Raylib_IsCursorHidden);
    register_callback(collector_context, c"Raylib::EnableCursor".as_ptr(), Raylib_EnableCursor);
    register_callback(collector_context, c"Raylib::enable_cursor".as_ptr(), Raylib_EnableCursor);
    register_callback(collector_context, c"raylib::EnableCursor".as_ptr(), Raylib_EnableCursor);
    register_callback(collector_context, c"raylib::enable_cursor".as_ptr(), Raylib_EnableCursor);
    register_callback(collector_context, c"EnableCursor".as_ptr(), Raylib_EnableCursor);
    register_callback(collector_context, c"enable_cursor".as_ptr(), Raylib_EnableCursor);
    register_callback(collector_context, c"Raylib::DisableCursor".as_ptr(), Raylib_DisableCursor);
    register_callback(collector_context, c"Raylib::disable_cursor".as_ptr(), Raylib_DisableCursor);
    register_callback(collector_context, c"raylib::DisableCursor".as_ptr(), Raylib_DisableCursor);
    register_callback(collector_context, c"raylib::disable_cursor".as_ptr(), Raylib_DisableCursor);
    register_callback(collector_context, c"DisableCursor".as_ptr(), Raylib_DisableCursor);
    register_callback(collector_context, c"disable_cursor".as_ptr(), Raylib_DisableCursor);
    register_callback(collector_context, c"Raylib::IsCursorOnScreen".as_ptr(), Raylib_IsCursorOnScreen);
    register_callback(collector_context, c"Raylib::is_cursor_on_screen".as_ptr(), Raylib_IsCursorOnScreen);
    register_callback(collector_context, c"raylib::IsCursorOnScreen".as_ptr(), Raylib_IsCursorOnScreen);
    register_callback(collector_context, c"raylib::is_cursor_on_screen".as_ptr(), Raylib_IsCursorOnScreen);
    register_callback(collector_context, c"IsCursorOnScreen".as_ptr(), Raylib_IsCursorOnScreen);
    register_callback(collector_context, c"is_cursor_on_screen".as_ptr(), Raylib_IsCursorOnScreen);
    register_callback(collector_context, c"Raylib::ClearBackground".as_ptr(), Raylib_ClearBackground);
    register_callback(collector_context, c"Raylib::clear_background".as_ptr(), Raylib_ClearBackground);
    register_callback(collector_context, c"raylib::ClearBackground".as_ptr(), Raylib_ClearBackground);
    register_callback(collector_context, c"raylib::clear_background".as_ptr(), Raylib_ClearBackground);
    register_callback(collector_context, c"ClearBackground".as_ptr(), Raylib_ClearBackground);
    register_callback(collector_context, c"clear_background".as_ptr(), Raylib_ClearBackground);
    register_callback(collector_context, c"Raylib::BeginDrawing".as_ptr(), Raylib_BeginDrawing);
    register_callback(collector_context, c"Raylib::begin_drawing".as_ptr(), Raylib_BeginDrawing);
    register_callback(collector_context, c"raylib::BeginDrawing".as_ptr(), Raylib_BeginDrawing);
    register_callback(collector_context, c"raylib::begin_drawing".as_ptr(), Raylib_BeginDrawing);
    register_callback(collector_context, c"BeginDrawing".as_ptr(), Raylib_BeginDrawing);
    register_callback(collector_context, c"begin_drawing".as_ptr(), Raylib_BeginDrawing);
    register_callback(collector_context, c"Raylib::EndDrawing".as_ptr(), Raylib_EndDrawing);
    register_callback(collector_context, c"Raylib::end_drawing".as_ptr(), Raylib_EndDrawing);
    register_callback(collector_context, c"raylib::EndDrawing".as_ptr(), Raylib_EndDrawing);
    register_callback(collector_context, c"raylib::end_drawing".as_ptr(), Raylib_EndDrawing);
    register_callback(collector_context, c"EndDrawing".as_ptr(), Raylib_EndDrawing);
    register_callback(collector_context, c"end_drawing".as_ptr(), Raylib_EndDrawing);
    register_callback(collector_context, c"Raylib::BeginMode2D".as_ptr(), Raylib_BeginMode2D);
    register_callback(collector_context, c"Raylib::begin_mode2d".as_ptr(), Raylib_BeginMode2D);
    register_callback(collector_context, c"raylib::BeginMode2D".as_ptr(), Raylib_BeginMode2D);
    register_callback(collector_context, c"raylib::begin_mode2d".as_ptr(), Raylib_BeginMode2D);
    register_callback(collector_context, c"BeginMode2D".as_ptr(), Raylib_BeginMode2D);
    register_callback(collector_context, c"begin_mode2d".as_ptr(), Raylib_BeginMode2D);
    register_callback(collector_context, c"Raylib::EndMode2D".as_ptr(), Raylib_EndMode2D);
    register_callback(collector_context, c"Raylib::end_mode2d".as_ptr(), Raylib_EndMode2D);
    register_callback(collector_context, c"raylib::EndMode2D".as_ptr(), Raylib_EndMode2D);
    register_callback(collector_context, c"raylib::end_mode2d".as_ptr(), Raylib_EndMode2D);
    register_callback(collector_context, c"EndMode2D".as_ptr(), Raylib_EndMode2D);
    register_callback(collector_context, c"end_mode2d".as_ptr(), Raylib_EndMode2D);
    register_callback(collector_context, c"Raylib::BeginMode3D".as_ptr(), Raylib_BeginMode3D);
    register_callback(collector_context, c"Raylib::begin_mode3d".as_ptr(), Raylib_BeginMode3D);
    register_callback(collector_context, c"raylib::BeginMode3D".as_ptr(), Raylib_BeginMode3D);
    register_callback(collector_context, c"raylib::begin_mode3d".as_ptr(), Raylib_BeginMode3D);
    register_callback(collector_context, c"BeginMode3D".as_ptr(), Raylib_BeginMode3D);
    register_callback(collector_context, c"begin_mode3d".as_ptr(), Raylib_BeginMode3D);
    register_callback(collector_context, c"Raylib::EndMode3D".as_ptr(), Raylib_EndMode3D);
    register_callback(collector_context, c"Raylib::end_mode3d".as_ptr(), Raylib_EndMode3D);
    register_callback(collector_context, c"raylib::EndMode3D".as_ptr(), Raylib_EndMode3D);
    register_callback(collector_context, c"raylib::end_mode3d".as_ptr(), Raylib_EndMode3D);
    register_callback(collector_context, c"EndMode3D".as_ptr(), Raylib_EndMode3D);
    register_callback(collector_context, c"end_mode3d".as_ptr(), Raylib_EndMode3D);
    register_callback(collector_context, c"Raylib::BeginTextureMode".as_ptr(), Raylib_BeginTextureMode);
    register_callback(collector_context, c"Raylib::begin_texture_mode".as_ptr(), Raylib_BeginTextureMode);
    register_callback(collector_context, c"raylib::BeginTextureMode".as_ptr(), Raylib_BeginTextureMode);
    register_callback(collector_context, c"raylib::begin_texture_mode".as_ptr(), Raylib_BeginTextureMode);
    register_callback(collector_context, c"BeginTextureMode".as_ptr(), Raylib_BeginTextureMode);
    register_callback(collector_context, c"begin_texture_mode".as_ptr(), Raylib_BeginTextureMode);
    register_callback(collector_context, c"Raylib::EndTextureMode".as_ptr(), Raylib_EndTextureMode);
    register_callback(collector_context, c"Raylib::end_texture_mode".as_ptr(), Raylib_EndTextureMode);
    register_callback(collector_context, c"raylib::EndTextureMode".as_ptr(), Raylib_EndTextureMode);
    register_callback(collector_context, c"raylib::end_texture_mode".as_ptr(), Raylib_EndTextureMode);
    register_callback(collector_context, c"EndTextureMode".as_ptr(), Raylib_EndTextureMode);
    register_callback(collector_context, c"end_texture_mode".as_ptr(), Raylib_EndTextureMode);
    register_callback(collector_context, c"Raylib::BeginShaderMode".as_ptr(), Raylib_BeginShaderMode);
    register_callback(collector_context, c"Raylib::begin_shader_mode".as_ptr(), Raylib_BeginShaderMode);
    register_callback(collector_context, c"raylib::BeginShaderMode".as_ptr(), Raylib_BeginShaderMode);
    register_callback(collector_context, c"raylib::begin_shader_mode".as_ptr(), Raylib_BeginShaderMode);
    register_callback(collector_context, c"BeginShaderMode".as_ptr(), Raylib_BeginShaderMode);
    register_callback(collector_context, c"begin_shader_mode".as_ptr(), Raylib_BeginShaderMode);
    register_callback(collector_context, c"Raylib::EndShaderMode".as_ptr(), Raylib_EndShaderMode);
    register_callback(collector_context, c"Raylib::end_shader_mode".as_ptr(), Raylib_EndShaderMode);
    register_callback(collector_context, c"raylib::EndShaderMode".as_ptr(), Raylib_EndShaderMode);
    register_callback(collector_context, c"raylib::end_shader_mode".as_ptr(), Raylib_EndShaderMode);
    register_callback(collector_context, c"EndShaderMode".as_ptr(), Raylib_EndShaderMode);
    register_callback(collector_context, c"end_shader_mode".as_ptr(), Raylib_EndShaderMode);
    register_callback(collector_context, c"Raylib::BeginBlendMode".as_ptr(), Raylib_BeginBlendMode);
    register_callback(collector_context, c"Raylib::begin_blend_mode".as_ptr(), Raylib_BeginBlendMode);
    register_callback(collector_context, c"raylib::BeginBlendMode".as_ptr(), Raylib_BeginBlendMode);
    register_callback(collector_context, c"raylib::begin_blend_mode".as_ptr(), Raylib_BeginBlendMode);
    register_callback(collector_context, c"BeginBlendMode".as_ptr(), Raylib_BeginBlendMode);
    register_callback(collector_context, c"begin_blend_mode".as_ptr(), Raylib_BeginBlendMode);
    register_callback(collector_context, c"Raylib::EndBlendMode".as_ptr(), Raylib_EndBlendMode);
    register_callback(collector_context, c"Raylib::end_blend_mode".as_ptr(), Raylib_EndBlendMode);
    register_callback(collector_context, c"raylib::EndBlendMode".as_ptr(), Raylib_EndBlendMode);
    register_callback(collector_context, c"raylib::end_blend_mode".as_ptr(), Raylib_EndBlendMode);
    register_callback(collector_context, c"EndBlendMode".as_ptr(), Raylib_EndBlendMode);
    register_callback(collector_context, c"end_blend_mode".as_ptr(), Raylib_EndBlendMode);
    register_callback(collector_context, c"Raylib::BeginScissorMode".as_ptr(), Raylib_BeginScissorMode);
    register_callback(collector_context, c"Raylib::begin_scissor_mode".as_ptr(), Raylib_BeginScissorMode);
    register_callback(collector_context, c"raylib::BeginScissorMode".as_ptr(), Raylib_BeginScissorMode);
    register_callback(collector_context, c"raylib::begin_scissor_mode".as_ptr(), Raylib_BeginScissorMode);
    register_callback(collector_context, c"BeginScissorMode".as_ptr(), Raylib_BeginScissorMode);
    register_callback(collector_context, c"begin_scissor_mode".as_ptr(), Raylib_BeginScissorMode);
    register_callback(collector_context, c"Raylib::EndScissorMode".as_ptr(), Raylib_EndScissorMode);
    register_callback(collector_context, c"Raylib::end_scissor_mode".as_ptr(), Raylib_EndScissorMode);
    register_callback(collector_context, c"raylib::EndScissorMode".as_ptr(), Raylib_EndScissorMode);
    register_callback(collector_context, c"raylib::end_scissor_mode".as_ptr(), Raylib_EndScissorMode);
    register_callback(collector_context, c"EndScissorMode".as_ptr(), Raylib_EndScissorMode);
    register_callback(collector_context, c"end_scissor_mode".as_ptr(), Raylib_EndScissorMode);
    register_callback(collector_context, c"Raylib::BeginVrStereoMode".as_ptr(), Raylib_BeginVrStereoMode);
    register_callback(collector_context, c"Raylib::begin_vr_stereo_mode".as_ptr(), Raylib_BeginVrStereoMode);
    register_callback(collector_context, c"raylib::BeginVrStereoMode".as_ptr(), Raylib_BeginVrStereoMode);
    register_callback(collector_context, c"raylib::begin_vr_stereo_mode".as_ptr(), Raylib_BeginVrStereoMode);
    register_callback(collector_context, c"BeginVrStereoMode".as_ptr(), Raylib_BeginVrStereoMode);
    register_callback(collector_context, c"begin_vr_stereo_mode".as_ptr(), Raylib_BeginVrStereoMode);
    register_callback(collector_context, c"Raylib::EndVrStereoMode".as_ptr(), Raylib_EndVrStereoMode);
    register_callback(collector_context, c"Raylib::end_vr_stereo_mode".as_ptr(), Raylib_EndVrStereoMode);
    register_callback(collector_context, c"raylib::EndVrStereoMode".as_ptr(), Raylib_EndVrStereoMode);
    register_callback(collector_context, c"raylib::end_vr_stereo_mode".as_ptr(), Raylib_EndVrStereoMode);
    register_callback(collector_context, c"EndVrStereoMode".as_ptr(), Raylib_EndVrStereoMode);
    register_callback(collector_context, c"end_vr_stereo_mode".as_ptr(), Raylib_EndVrStereoMode);
    register_callback(collector_context, c"Raylib::LoadVrStereoConfig".as_ptr(), Raylib_LoadVrStereoConfig);
    register_callback(collector_context, c"Raylib::load_vr_stereo_config".as_ptr(), Raylib_LoadVrStereoConfig);
    register_callback(collector_context, c"raylib::LoadVrStereoConfig".as_ptr(), Raylib_LoadVrStereoConfig);
    register_callback(collector_context, c"raylib::load_vr_stereo_config".as_ptr(), Raylib_LoadVrStereoConfig);
    register_callback(collector_context, c"LoadVrStereoConfig".as_ptr(), Raylib_LoadVrStereoConfig);
    register_callback(collector_context, c"load_vr_stereo_config".as_ptr(), Raylib_LoadVrStereoConfig);
    register_callback(collector_context, c"Raylib::UnloadVrStereoConfig".as_ptr(), Raylib_UnloadVrStereoConfig);
    register_callback(collector_context, c"Raylib::unload_vr_stereo_config".as_ptr(), Raylib_UnloadVrStereoConfig);
    register_callback(collector_context, c"raylib::UnloadVrStereoConfig".as_ptr(), Raylib_UnloadVrStereoConfig);
    register_callback(collector_context, c"raylib::unload_vr_stereo_config".as_ptr(), Raylib_UnloadVrStereoConfig);
    register_callback(collector_context, c"UnloadVrStereoConfig".as_ptr(), Raylib_UnloadVrStereoConfig);
    register_callback(collector_context, c"unload_vr_stereo_config".as_ptr(), Raylib_UnloadVrStereoConfig);
    register_callback(collector_context, c"Raylib::LoadShader".as_ptr(), Raylib_LoadShader);
    register_callback(collector_context, c"Raylib::load_shader".as_ptr(), Raylib_LoadShader);
    register_callback(collector_context, c"raylib::LoadShader".as_ptr(), Raylib_LoadShader);
    register_callback(collector_context, c"raylib::load_shader".as_ptr(), Raylib_LoadShader);
    register_callback(collector_context, c"LoadShader".as_ptr(), Raylib_LoadShader);
    register_callback(collector_context, c"load_shader".as_ptr(), Raylib_LoadShader);
    register_callback(collector_context, c"Raylib::LoadShaderFromMemory".as_ptr(), Raylib_LoadShaderFromMemory);
    register_callback(collector_context, c"Raylib::load_shader_from_memory".as_ptr(), Raylib_LoadShaderFromMemory);
    register_callback(collector_context, c"raylib::LoadShaderFromMemory".as_ptr(), Raylib_LoadShaderFromMemory);
    register_callback(collector_context, c"raylib::load_shader_from_memory".as_ptr(), Raylib_LoadShaderFromMemory);
    register_callback(collector_context, c"LoadShaderFromMemory".as_ptr(), Raylib_LoadShaderFromMemory);
    register_callback(collector_context, c"load_shader_from_memory".as_ptr(), Raylib_LoadShaderFromMemory);
    register_callback(collector_context, c"Raylib::IsShaderValid".as_ptr(), Raylib_IsShaderValid);
    register_callback(collector_context, c"Raylib::is_shader_valid".as_ptr(), Raylib_IsShaderValid);
    register_callback(collector_context, c"raylib::IsShaderValid".as_ptr(), Raylib_IsShaderValid);
    register_callback(collector_context, c"raylib::is_shader_valid".as_ptr(), Raylib_IsShaderValid);
    register_callback(collector_context, c"IsShaderValid".as_ptr(), Raylib_IsShaderValid);
    register_callback(collector_context, c"is_shader_valid".as_ptr(), Raylib_IsShaderValid);
    register_callback(collector_context, c"Raylib::GetShaderLocation".as_ptr(), Raylib_GetShaderLocation);
    register_callback(collector_context, c"Raylib::get_shader_location".as_ptr(), Raylib_GetShaderLocation);
    register_callback(collector_context, c"raylib::GetShaderLocation".as_ptr(), Raylib_GetShaderLocation);
    register_callback(collector_context, c"raylib::get_shader_location".as_ptr(), Raylib_GetShaderLocation);
    register_callback(collector_context, c"GetShaderLocation".as_ptr(), Raylib_GetShaderLocation);
    register_callback(collector_context, c"get_shader_location".as_ptr(), Raylib_GetShaderLocation);
    register_callback(collector_context, c"Raylib::GetShaderLocationAttrib".as_ptr(), Raylib_GetShaderLocationAttrib);
    register_callback(collector_context, c"Raylib::get_shader_location_attrib".as_ptr(), Raylib_GetShaderLocationAttrib);
    register_callback(collector_context, c"raylib::GetShaderLocationAttrib".as_ptr(), Raylib_GetShaderLocationAttrib);
    register_callback(collector_context, c"raylib::get_shader_location_attrib".as_ptr(), Raylib_GetShaderLocationAttrib);
    register_callback(collector_context, c"GetShaderLocationAttrib".as_ptr(), Raylib_GetShaderLocationAttrib);
    register_callback(collector_context, c"get_shader_location_attrib".as_ptr(), Raylib_GetShaderLocationAttrib);
    register_callback(collector_context, c"Raylib::SetShaderValue".as_ptr(), Raylib_SetShaderValue);
    register_callback(collector_context, c"Raylib::set_shader_value".as_ptr(), Raylib_SetShaderValue);
    register_callback(collector_context, c"raylib::SetShaderValue".as_ptr(), Raylib_SetShaderValue);
    register_callback(collector_context, c"raylib::set_shader_value".as_ptr(), Raylib_SetShaderValue);
    register_callback(collector_context, c"SetShaderValue".as_ptr(), Raylib_SetShaderValue);
    register_callback(collector_context, c"set_shader_value".as_ptr(), Raylib_SetShaderValue);
    register_callback(collector_context, c"Raylib::SetShaderValueV".as_ptr(), Raylib_SetShaderValueV);
    register_callback(collector_context, c"Raylib::set_shader_value_v".as_ptr(), Raylib_SetShaderValueV);
    register_callback(collector_context, c"raylib::SetShaderValueV".as_ptr(), Raylib_SetShaderValueV);
    register_callback(collector_context, c"raylib::set_shader_value_v".as_ptr(), Raylib_SetShaderValueV);
    register_callback(collector_context, c"SetShaderValueV".as_ptr(), Raylib_SetShaderValueV);
    register_callback(collector_context, c"set_shader_value_v".as_ptr(), Raylib_SetShaderValueV);
    register_callback(collector_context, c"Raylib::SetShaderValueMatrix".as_ptr(), Raylib_SetShaderValueMatrix);
    register_callback(collector_context, c"Raylib::set_shader_value_matrix".as_ptr(), Raylib_SetShaderValueMatrix);
    register_callback(collector_context, c"raylib::SetShaderValueMatrix".as_ptr(), Raylib_SetShaderValueMatrix);
    register_callback(collector_context, c"raylib::set_shader_value_matrix".as_ptr(), Raylib_SetShaderValueMatrix);
    register_callback(collector_context, c"SetShaderValueMatrix".as_ptr(), Raylib_SetShaderValueMatrix);
    register_callback(collector_context, c"set_shader_value_matrix".as_ptr(), Raylib_SetShaderValueMatrix);
    register_callback(collector_context, c"Raylib::SetShaderValueTexture".as_ptr(), Raylib_SetShaderValueTexture);
    register_callback(collector_context, c"Raylib::set_shader_value_texture".as_ptr(), Raylib_SetShaderValueTexture);
    register_callback(collector_context, c"raylib::SetShaderValueTexture".as_ptr(), Raylib_SetShaderValueTexture);
    register_callback(collector_context, c"raylib::set_shader_value_texture".as_ptr(), Raylib_SetShaderValueTexture);
    register_callback(collector_context, c"SetShaderValueTexture".as_ptr(), Raylib_SetShaderValueTexture);
    register_callback(collector_context, c"set_shader_value_texture".as_ptr(), Raylib_SetShaderValueTexture);
    register_callback(collector_context, c"Raylib::UnloadShader".as_ptr(), Raylib_UnloadShader);
    register_callback(collector_context, c"Raylib::unload_shader".as_ptr(), Raylib_UnloadShader);
    register_callback(collector_context, c"raylib::UnloadShader".as_ptr(), Raylib_UnloadShader);
    register_callback(collector_context, c"raylib::unload_shader".as_ptr(), Raylib_UnloadShader);
    register_callback(collector_context, c"UnloadShader".as_ptr(), Raylib_UnloadShader);
    register_callback(collector_context, c"unload_shader".as_ptr(), Raylib_UnloadShader);
    register_callback(collector_context, c"Raylib::GetScreenToWorldRay".as_ptr(), Raylib_GetScreenToWorldRay);
    register_callback(collector_context, c"Raylib::get_screen_to_world_ray".as_ptr(), Raylib_GetScreenToWorldRay);
    register_callback(collector_context, c"raylib::GetScreenToWorldRay".as_ptr(), Raylib_GetScreenToWorldRay);
    register_callback(collector_context, c"raylib::get_screen_to_world_ray".as_ptr(), Raylib_GetScreenToWorldRay);
    register_callback(collector_context, c"GetScreenToWorldRay".as_ptr(), Raylib_GetScreenToWorldRay);
    register_callback(collector_context, c"get_screen_to_world_ray".as_ptr(), Raylib_GetScreenToWorldRay);
    register_callback(collector_context, c"Raylib::GetScreenToWorldRayEx".as_ptr(), Raylib_GetScreenToWorldRayEx);
    register_callback(collector_context, c"Raylib::get_screen_to_world_ray_ex".as_ptr(), Raylib_GetScreenToWorldRayEx);
    register_callback(collector_context, c"raylib::GetScreenToWorldRayEx".as_ptr(), Raylib_GetScreenToWorldRayEx);
    register_callback(collector_context, c"raylib::get_screen_to_world_ray_ex".as_ptr(), Raylib_GetScreenToWorldRayEx);
    register_callback(collector_context, c"GetScreenToWorldRayEx".as_ptr(), Raylib_GetScreenToWorldRayEx);
    register_callback(collector_context, c"get_screen_to_world_ray_ex".as_ptr(), Raylib_GetScreenToWorldRayEx);
    register_callback(collector_context, c"Raylib::GetWorldToScreen".as_ptr(), Raylib_GetWorldToScreen);
    register_callback(collector_context, c"Raylib::get_world_to_screen".as_ptr(), Raylib_GetWorldToScreen);
    register_callback(collector_context, c"raylib::GetWorldToScreen".as_ptr(), Raylib_GetWorldToScreen);
    register_callback(collector_context, c"raylib::get_world_to_screen".as_ptr(), Raylib_GetWorldToScreen);
    register_callback(collector_context, c"GetWorldToScreen".as_ptr(), Raylib_GetWorldToScreen);
    register_callback(collector_context, c"get_world_to_screen".as_ptr(), Raylib_GetWorldToScreen);
    register_callback(collector_context, c"Raylib::GetWorldToScreenEx".as_ptr(), Raylib_GetWorldToScreenEx);
    register_callback(collector_context, c"Raylib::get_world_to_screen_ex".as_ptr(), Raylib_GetWorldToScreenEx);
    register_callback(collector_context, c"raylib::GetWorldToScreenEx".as_ptr(), Raylib_GetWorldToScreenEx);
    register_callback(collector_context, c"raylib::get_world_to_screen_ex".as_ptr(), Raylib_GetWorldToScreenEx);
    register_callback(collector_context, c"GetWorldToScreenEx".as_ptr(), Raylib_GetWorldToScreenEx);
    register_callback(collector_context, c"get_world_to_screen_ex".as_ptr(), Raylib_GetWorldToScreenEx);
    register_callback(collector_context, c"Raylib::GetWorldToScreen2D".as_ptr(), Raylib_GetWorldToScreen2D);
    register_callback(collector_context, c"Raylib::get_world_to_screen2d".as_ptr(), Raylib_GetWorldToScreen2D);
    register_callback(collector_context, c"raylib::GetWorldToScreen2D".as_ptr(), Raylib_GetWorldToScreen2D);
    register_callback(collector_context, c"raylib::get_world_to_screen2d".as_ptr(), Raylib_GetWorldToScreen2D);
    register_callback(collector_context, c"GetWorldToScreen2D".as_ptr(), Raylib_GetWorldToScreen2D);
    register_callback(collector_context, c"get_world_to_screen2d".as_ptr(), Raylib_GetWorldToScreen2D);
    register_callback(collector_context, c"Raylib::GetScreenToWorld2D".as_ptr(), Raylib_GetScreenToWorld2D);
    register_callback(collector_context, c"Raylib::get_screen_to_world2d".as_ptr(), Raylib_GetScreenToWorld2D);
    register_callback(collector_context, c"raylib::GetScreenToWorld2D".as_ptr(), Raylib_GetScreenToWorld2D);
    register_callback(collector_context, c"raylib::get_screen_to_world2d".as_ptr(), Raylib_GetScreenToWorld2D);
    register_callback(collector_context, c"GetScreenToWorld2D".as_ptr(), Raylib_GetScreenToWorld2D);
    register_callback(collector_context, c"get_screen_to_world2d".as_ptr(), Raylib_GetScreenToWorld2D);
    register_callback(collector_context, c"Raylib::GetCameraMatrix".as_ptr(), Raylib_GetCameraMatrix);
    register_callback(collector_context, c"Raylib::get_camera_matrix".as_ptr(), Raylib_GetCameraMatrix);
    register_callback(collector_context, c"raylib::GetCameraMatrix".as_ptr(), Raylib_GetCameraMatrix);
    register_callback(collector_context, c"raylib::get_camera_matrix".as_ptr(), Raylib_GetCameraMatrix);
    register_callback(collector_context, c"GetCameraMatrix".as_ptr(), Raylib_GetCameraMatrix);
    register_callback(collector_context, c"get_camera_matrix".as_ptr(), Raylib_GetCameraMatrix);
    register_callback(collector_context, c"Raylib::GetCameraMatrix2D".as_ptr(), Raylib_GetCameraMatrix2D);
    register_callback(collector_context, c"Raylib::get_camera_matrix2d".as_ptr(), Raylib_GetCameraMatrix2D);
    register_callback(collector_context, c"raylib::GetCameraMatrix2D".as_ptr(), Raylib_GetCameraMatrix2D);
    register_callback(collector_context, c"raylib::get_camera_matrix2d".as_ptr(), Raylib_GetCameraMatrix2D);
    register_callback(collector_context, c"GetCameraMatrix2D".as_ptr(), Raylib_GetCameraMatrix2D);
    register_callback(collector_context, c"get_camera_matrix2d".as_ptr(), Raylib_GetCameraMatrix2D);
    register_callback(collector_context, c"Raylib::SetTargetFPS".as_ptr(), Raylib_SetTargetFPS);
    register_callback(collector_context, c"Raylib::set_target_fps".as_ptr(), Raylib_SetTargetFPS);
    register_callback(collector_context, c"raylib::SetTargetFPS".as_ptr(), Raylib_SetTargetFPS);
    register_callback(collector_context, c"raylib::set_target_fps".as_ptr(), Raylib_SetTargetFPS);
    register_callback(collector_context, c"SetTargetFPS".as_ptr(), Raylib_SetTargetFPS);
    register_callback(collector_context, c"set_target_fps".as_ptr(), Raylib_SetTargetFPS);
    register_callback(collector_context, c"Raylib::GetFrameTime".as_ptr(), Raylib_GetFrameTime);
    register_callback(collector_context, c"Raylib::get_frame_time".as_ptr(), Raylib_GetFrameTime);
    register_callback(collector_context, c"raylib::GetFrameTime".as_ptr(), Raylib_GetFrameTime);
    register_callback(collector_context, c"raylib::get_frame_time".as_ptr(), Raylib_GetFrameTime);
    register_callback(collector_context, c"GetFrameTime".as_ptr(), Raylib_GetFrameTime);
    register_callback(collector_context, c"get_frame_time".as_ptr(), Raylib_GetFrameTime);
    register_callback(collector_context, c"Raylib::GetTime".as_ptr(), Raylib_GetTime);
    register_callback(collector_context, c"Raylib::get_time".as_ptr(), Raylib_GetTime);
    register_callback(collector_context, c"raylib::GetTime".as_ptr(), Raylib_GetTime);
    register_callback(collector_context, c"raylib::get_time".as_ptr(), Raylib_GetTime);
    register_callback(collector_context, c"GetTime".as_ptr(), Raylib_GetTime);
    register_callback(collector_context, c"get_time".as_ptr(), Raylib_GetTime);
    register_callback(collector_context, c"Raylib::GetFPS".as_ptr(), Raylib_GetFPS);
    register_callback(collector_context, c"Raylib::get_fps".as_ptr(), Raylib_GetFPS);
    register_callback(collector_context, c"raylib::GetFPS".as_ptr(), Raylib_GetFPS);
    register_callback(collector_context, c"raylib::get_fps".as_ptr(), Raylib_GetFPS);
    register_callback(collector_context, c"GetFPS".as_ptr(), Raylib_GetFPS);
    register_callback(collector_context, c"get_fps".as_ptr(), Raylib_GetFPS);
    register_callback(collector_context, c"Raylib::SwapScreenBuffer".as_ptr(), Raylib_SwapScreenBuffer);
    register_callback(collector_context, c"Raylib::swap_screen_buffer".as_ptr(), Raylib_SwapScreenBuffer);
    register_callback(collector_context, c"raylib::SwapScreenBuffer".as_ptr(), Raylib_SwapScreenBuffer);
    register_callback(collector_context, c"raylib::swap_screen_buffer".as_ptr(), Raylib_SwapScreenBuffer);
    register_callback(collector_context, c"SwapScreenBuffer".as_ptr(), Raylib_SwapScreenBuffer);
    register_callback(collector_context, c"swap_screen_buffer".as_ptr(), Raylib_SwapScreenBuffer);
    register_callback(collector_context, c"Raylib::PollInputEvents".as_ptr(), Raylib_PollInputEvents);
    register_callback(collector_context, c"Raylib::poll_input_events".as_ptr(), Raylib_PollInputEvents);
    register_callback(collector_context, c"raylib::PollInputEvents".as_ptr(), Raylib_PollInputEvents);
    register_callback(collector_context, c"raylib::poll_input_events".as_ptr(), Raylib_PollInputEvents);
    register_callback(collector_context, c"PollInputEvents".as_ptr(), Raylib_PollInputEvents);
    register_callback(collector_context, c"poll_input_events".as_ptr(), Raylib_PollInputEvents);
    register_callback(collector_context, c"Raylib::WaitTime".as_ptr(), Raylib_WaitTime);
    register_callback(collector_context, c"Raylib::wait_time".as_ptr(), Raylib_WaitTime);
    register_callback(collector_context, c"raylib::WaitTime".as_ptr(), Raylib_WaitTime);
    register_callback(collector_context, c"raylib::wait_time".as_ptr(), Raylib_WaitTime);
    register_callback(collector_context, c"WaitTime".as_ptr(), Raylib_WaitTime);
    register_callback(collector_context, c"wait_time".as_ptr(), Raylib_WaitTime);
    register_callback(collector_context, c"Raylib::SetRandomSeed".as_ptr(), Raylib_SetRandomSeed);
    register_callback(collector_context, c"Raylib::set_random_seed".as_ptr(), Raylib_SetRandomSeed);
    register_callback(collector_context, c"raylib::SetRandomSeed".as_ptr(), Raylib_SetRandomSeed);
    register_callback(collector_context, c"raylib::set_random_seed".as_ptr(), Raylib_SetRandomSeed);
    register_callback(collector_context, c"SetRandomSeed".as_ptr(), Raylib_SetRandomSeed);
    register_callback(collector_context, c"set_random_seed".as_ptr(), Raylib_SetRandomSeed);
    register_callback(collector_context, c"Raylib::GetRandomValue".as_ptr(), Raylib_GetRandomValue);
    register_callback(collector_context, c"Raylib::get_random_value".as_ptr(), Raylib_GetRandomValue);
    register_callback(collector_context, c"raylib::GetRandomValue".as_ptr(), Raylib_GetRandomValue);
    register_callback(collector_context, c"raylib::get_random_value".as_ptr(), Raylib_GetRandomValue);
    register_callback(collector_context, c"GetRandomValue".as_ptr(), Raylib_GetRandomValue);
    register_callback(collector_context, c"get_random_value".as_ptr(), Raylib_GetRandomValue);
    register_callback(collector_context, c"Raylib::LoadRandomSequence".as_ptr(), Raylib_LoadRandomSequence);
    register_callback(collector_context, c"Raylib::load_random_sequence".as_ptr(), Raylib_LoadRandomSequence);
    register_callback(collector_context, c"raylib::LoadRandomSequence".as_ptr(), Raylib_LoadRandomSequence);
    register_callback(collector_context, c"raylib::load_random_sequence".as_ptr(), Raylib_LoadRandomSequence);
    register_callback(collector_context, c"LoadRandomSequence".as_ptr(), Raylib_LoadRandomSequence);
    register_callback(collector_context, c"load_random_sequence".as_ptr(), Raylib_LoadRandomSequence);
    register_callback(collector_context, c"Raylib::UnloadRandomSequence".as_ptr(), Raylib_UnloadRandomSequence);
    register_callback(collector_context, c"Raylib::unload_random_sequence".as_ptr(), Raylib_UnloadRandomSequence);
    register_callback(collector_context, c"raylib::UnloadRandomSequence".as_ptr(), Raylib_UnloadRandomSequence);
    register_callback(collector_context, c"raylib::unload_random_sequence".as_ptr(), Raylib_UnloadRandomSequence);
    register_callback(collector_context, c"UnloadRandomSequence".as_ptr(), Raylib_UnloadRandomSequence);
    register_callback(collector_context, c"unload_random_sequence".as_ptr(), Raylib_UnloadRandomSequence);
    register_callback(collector_context, c"Raylib::TakeScreenshot".as_ptr(), Raylib_TakeScreenshot);
    register_callback(collector_context, c"Raylib::take_screenshot".as_ptr(), Raylib_TakeScreenshot);
    register_callback(collector_context, c"raylib::TakeScreenshot".as_ptr(), Raylib_TakeScreenshot);
    register_callback(collector_context, c"raylib::take_screenshot".as_ptr(), Raylib_TakeScreenshot);
    register_callback(collector_context, c"TakeScreenshot".as_ptr(), Raylib_TakeScreenshot);
    register_callback(collector_context, c"take_screenshot".as_ptr(), Raylib_TakeScreenshot);
    register_callback(collector_context, c"Raylib::SetConfigFlags".as_ptr(), Raylib_SetConfigFlags);
    register_callback(collector_context, c"Raylib::set_config_flags".as_ptr(), Raylib_SetConfigFlags);
    register_callback(collector_context, c"raylib::SetConfigFlags".as_ptr(), Raylib_SetConfigFlags);
    register_callback(collector_context, c"raylib::set_config_flags".as_ptr(), Raylib_SetConfigFlags);
    register_callback(collector_context, c"SetConfigFlags".as_ptr(), Raylib_SetConfigFlags);
    register_callback(collector_context, c"set_config_flags".as_ptr(), Raylib_SetConfigFlags);
    register_callback(collector_context, c"Raylib::OpenURL".as_ptr(), Raylib_OpenURL);
    register_callback(collector_context, c"Raylib::open_url".as_ptr(), Raylib_OpenURL);
    register_callback(collector_context, c"raylib::OpenURL".as_ptr(), Raylib_OpenURL);
    register_callback(collector_context, c"raylib::open_url".as_ptr(), Raylib_OpenURL);
    register_callback(collector_context, c"OpenURL".as_ptr(), Raylib_OpenURL);
    register_callback(collector_context, c"open_url".as_ptr(), Raylib_OpenURL);
    register_callback(collector_context, c"Raylib::SetTraceLogLevel".as_ptr(), Raylib_SetTraceLogLevel);
    register_callback(collector_context, c"Raylib::set_trace_log_level".as_ptr(), Raylib_SetTraceLogLevel);
    register_callback(collector_context, c"raylib::SetTraceLogLevel".as_ptr(), Raylib_SetTraceLogLevel);
    register_callback(collector_context, c"raylib::set_trace_log_level".as_ptr(), Raylib_SetTraceLogLevel);
    register_callback(collector_context, c"SetTraceLogLevel".as_ptr(), Raylib_SetTraceLogLevel);
    register_callback(collector_context, c"set_trace_log_level".as_ptr(), Raylib_SetTraceLogLevel);
    register_callback(collector_context, c"Raylib::TraceLog".as_ptr(), Raylib_TraceLog);
    register_callback(collector_context, c"Raylib::trace_log".as_ptr(), Raylib_TraceLog);
    register_callback(collector_context, c"raylib::TraceLog".as_ptr(), Raylib_TraceLog);
    register_callback(collector_context, c"raylib::trace_log".as_ptr(), Raylib_TraceLog);
    register_callback(collector_context, c"TraceLog".as_ptr(), Raylib_TraceLog);
    register_callback(collector_context, c"trace_log".as_ptr(), Raylib_TraceLog);
    register_callback(collector_context, c"Raylib::SetTraceLogCallback".as_ptr(), Raylib_SetTraceLogCallback);
    register_callback(collector_context, c"Raylib::set_trace_log_callback".as_ptr(), Raylib_SetTraceLogCallback);
    register_callback(collector_context, c"raylib::SetTraceLogCallback".as_ptr(), Raylib_SetTraceLogCallback);
    register_callback(collector_context, c"raylib::set_trace_log_callback".as_ptr(), Raylib_SetTraceLogCallback);
    register_callback(collector_context, c"SetTraceLogCallback".as_ptr(), Raylib_SetTraceLogCallback);
    register_callback(collector_context, c"set_trace_log_callback".as_ptr(), Raylib_SetTraceLogCallback);
    register_callback(collector_context, c"Raylib::MemAlloc".as_ptr(), Raylib_MemAlloc);
    register_callback(collector_context, c"Raylib::mem_alloc".as_ptr(), Raylib_MemAlloc);
    register_callback(collector_context, c"raylib::MemAlloc".as_ptr(), Raylib_MemAlloc);
    register_callback(collector_context, c"raylib::mem_alloc".as_ptr(), Raylib_MemAlloc);
    register_callback(collector_context, c"MemAlloc".as_ptr(), Raylib_MemAlloc);
    register_callback(collector_context, c"mem_alloc".as_ptr(), Raylib_MemAlloc);
    register_callback(collector_context, c"Raylib::MemRealloc".as_ptr(), Raylib_MemRealloc);
    register_callback(collector_context, c"Raylib::mem_realloc".as_ptr(), Raylib_MemRealloc);
    register_callback(collector_context, c"raylib::MemRealloc".as_ptr(), Raylib_MemRealloc);
    register_callback(collector_context, c"raylib::mem_realloc".as_ptr(), Raylib_MemRealloc);
    register_callback(collector_context, c"MemRealloc".as_ptr(), Raylib_MemRealloc);
    register_callback(collector_context, c"mem_realloc".as_ptr(), Raylib_MemRealloc);
    register_callback(collector_context, c"Raylib::MemFree".as_ptr(), Raylib_MemFree);
    register_callback(collector_context, c"Raylib::mem_free".as_ptr(), Raylib_MemFree);
    register_callback(collector_context, c"raylib::MemFree".as_ptr(), Raylib_MemFree);
    register_callback(collector_context, c"raylib::mem_free".as_ptr(), Raylib_MemFree);
    register_callback(collector_context, c"MemFree".as_ptr(), Raylib_MemFree);
    register_callback(collector_context, c"mem_free".as_ptr(), Raylib_MemFree);
    register_callback(collector_context, c"Raylib::LoadFileData".as_ptr(), Raylib_LoadFileData);
    register_callback(collector_context, c"Raylib::load_file_data".as_ptr(), Raylib_LoadFileData);
    register_callback(collector_context, c"raylib::LoadFileData".as_ptr(), Raylib_LoadFileData);
    register_callback(collector_context, c"raylib::load_file_data".as_ptr(), Raylib_LoadFileData);
    register_callback(collector_context, c"LoadFileData".as_ptr(), Raylib_LoadFileData);
    register_callback(collector_context, c"load_file_data".as_ptr(), Raylib_LoadFileData);
    register_callback(collector_context, c"Raylib::UnloadFileData".as_ptr(), Raylib_UnloadFileData);
    register_callback(collector_context, c"Raylib::unload_file_data".as_ptr(), Raylib_UnloadFileData);
    register_callback(collector_context, c"raylib::UnloadFileData".as_ptr(), Raylib_UnloadFileData);
    register_callback(collector_context, c"raylib::unload_file_data".as_ptr(), Raylib_UnloadFileData);
    register_callback(collector_context, c"UnloadFileData".as_ptr(), Raylib_UnloadFileData);
    register_callback(collector_context, c"unload_file_data".as_ptr(), Raylib_UnloadFileData);
    register_callback(collector_context, c"Raylib::SaveFileData".as_ptr(), Raylib_SaveFileData);
    register_callback(collector_context, c"Raylib::save_file_data".as_ptr(), Raylib_SaveFileData);
    register_callback(collector_context, c"raylib::SaveFileData".as_ptr(), Raylib_SaveFileData);
    register_callback(collector_context, c"raylib::save_file_data".as_ptr(), Raylib_SaveFileData);
    register_callback(collector_context, c"SaveFileData".as_ptr(), Raylib_SaveFileData);
    register_callback(collector_context, c"save_file_data".as_ptr(), Raylib_SaveFileData);
    register_callback(collector_context, c"Raylib::ExportDataAsCode".as_ptr(), Raylib_ExportDataAsCode);
    register_callback(collector_context, c"Raylib::export_data_as_code".as_ptr(), Raylib_ExportDataAsCode);
    register_callback(collector_context, c"raylib::ExportDataAsCode".as_ptr(), Raylib_ExportDataAsCode);
    register_callback(collector_context, c"raylib::export_data_as_code".as_ptr(), Raylib_ExportDataAsCode);
    register_callback(collector_context, c"ExportDataAsCode".as_ptr(), Raylib_ExportDataAsCode);
    register_callback(collector_context, c"export_data_as_code".as_ptr(), Raylib_ExportDataAsCode);
    register_callback(collector_context, c"Raylib::LoadFileText".as_ptr(), Raylib_LoadFileText);
    register_callback(collector_context, c"Raylib::load_file_text".as_ptr(), Raylib_LoadFileText);
    register_callback(collector_context, c"raylib::LoadFileText".as_ptr(), Raylib_LoadFileText);
    register_callback(collector_context, c"raylib::load_file_text".as_ptr(), Raylib_LoadFileText);
    register_callback(collector_context, c"LoadFileText".as_ptr(), Raylib_LoadFileText);
    register_callback(collector_context, c"load_file_text".as_ptr(), Raylib_LoadFileText);
    register_callback(collector_context, c"Raylib::UnloadFileText".as_ptr(), Raylib_UnloadFileText);
    register_callback(collector_context, c"Raylib::unload_file_text".as_ptr(), Raylib_UnloadFileText);
    register_callback(collector_context, c"raylib::UnloadFileText".as_ptr(), Raylib_UnloadFileText);
    register_callback(collector_context, c"raylib::unload_file_text".as_ptr(), Raylib_UnloadFileText);
    register_callback(collector_context, c"UnloadFileText".as_ptr(), Raylib_UnloadFileText);
    register_callback(collector_context, c"unload_file_text".as_ptr(), Raylib_UnloadFileText);
    register_callback(collector_context, c"Raylib::SaveFileText".as_ptr(), Raylib_SaveFileText);
    register_callback(collector_context, c"Raylib::save_file_text".as_ptr(), Raylib_SaveFileText);
    register_callback(collector_context, c"raylib::SaveFileText".as_ptr(), Raylib_SaveFileText);
    register_callback(collector_context, c"raylib::save_file_text".as_ptr(), Raylib_SaveFileText);
    register_callback(collector_context, c"SaveFileText".as_ptr(), Raylib_SaveFileText);
    register_callback(collector_context, c"save_file_text".as_ptr(), Raylib_SaveFileText);
    register_callback(collector_context, c"Raylib::SetLoadFileDataCallback".as_ptr(), Raylib_SetLoadFileDataCallback);
    register_callback(collector_context, c"Raylib::set_load_file_data_callback".as_ptr(), Raylib_SetLoadFileDataCallback);
    register_callback(collector_context, c"raylib::SetLoadFileDataCallback".as_ptr(), Raylib_SetLoadFileDataCallback);
    register_callback(collector_context, c"raylib::set_load_file_data_callback".as_ptr(), Raylib_SetLoadFileDataCallback);
    register_callback(collector_context, c"SetLoadFileDataCallback".as_ptr(), Raylib_SetLoadFileDataCallback);
    register_callback(collector_context, c"set_load_file_data_callback".as_ptr(), Raylib_SetLoadFileDataCallback);
    register_callback(collector_context, c"Raylib::SetSaveFileDataCallback".as_ptr(), Raylib_SetSaveFileDataCallback);
    register_callback(collector_context, c"Raylib::set_save_file_data_callback".as_ptr(), Raylib_SetSaveFileDataCallback);
    register_callback(collector_context, c"raylib::SetSaveFileDataCallback".as_ptr(), Raylib_SetSaveFileDataCallback);
    register_callback(collector_context, c"raylib::set_save_file_data_callback".as_ptr(), Raylib_SetSaveFileDataCallback);
    register_callback(collector_context, c"SetSaveFileDataCallback".as_ptr(), Raylib_SetSaveFileDataCallback);
    register_callback(collector_context, c"set_save_file_data_callback".as_ptr(), Raylib_SetSaveFileDataCallback);
    register_callback(collector_context, c"Raylib::SetLoadFileTextCallback".as_ptr(), Raylib_SetLoadFileTextCallback);
    register_callback(collector_context, c"Raylib::set_load_file_text_callback".as_ptr(), Raylib_SetLoadFileTextCallback);
    register_callback(collector_context, c"raylib::SetLoadFileTextCallback".as_ptr(), Raylib_SetLoadFileTextCallback);
    register_callback(collector_context, c"raylib::set_load_file_text_callback".as_ptr(), Raylib_SetLoadFileTextCallback);
    register_callback(collector_context, c"SetLoadFileTextCallback".as_ptr(), Raylib_SetLoadFileTextCallback);
    register_callback(collector_context, c"set_load_file_text_callback".as_ptr(), Raylib_SetLoadFileTextCallback);
    register_callback(collector_context, c"Raylib::SetSaveFileTextCallback".as_ptr(), Raylib_SetSaveFileTextCallback);
    register_callback(collector_context, c"Raylib::set_save_file_text_callback".as_ptr(), Raylib_SetSaveFileTextCallback);
    register_callback(collector_context, c"raylib::SetSaveFileTextCallback".as_ptr(), Raylib_SetSaveFileTextCallback);
    register_callback(collector_context, c"raylib::set_save_file_text_callback".as_ptr(), Raylib_SetSaveFileTextCallback);
    register_callback(collector_context, c"SetSaveFileTextCallback".as_ptr(), Raylib_SetSaveFileTextCallback);
    register_callback(collector_context, c"set_save_file_text_callback".as_ptr(), Raylib_SetSaveFileTextCallback);
    register_callback(collector_context, c"Raylib::FileRename".as_ptr(), Raylib_FileRename);
    register_callback(collector_context, c"Raylib::file_rename".as_ptr(), Raylib_FileRename);
    register_callback(collector_context, c"raylib::FileRename".as_ptr(), Raylib_FileRename);
    register_callback(collector_context, c"raylib::file_rename".as_ptr(), Raylib_FileRename);
    register_callback(collector_context, c"FileRename".as_ptr(), Raylib_FileRename);
    register_callback(collector_context, c"file_rename".as_ptr(), Raylib_FileRename);
    register_callback(collector_context, c"Raylib::FileRemove".as_ptr(), Raylib_FileRemove);
    register_callback(collector_context, c"Raylib::file_remove".as_ptr(), Raylib_FileRemove);
    register_callback(collector_context, c"raylib::FileRemove".as_ptr(), Raylib_FileRemove);
    register_callback(collector_context, c"raylib::file_remove".as_ptr(), Raylib_FileRemove);
    register_callback(collector_context, c"FileRemove".as_ptr(), Raylib_FileRemove);
    register_callback(collector_context, c"file_remove".as_ptr(), Raylib_FileRemove);
    register_callback(collector_context, c"Raylib::FileCopy".as_ptr(), Raylib_FileCopy);
    register_callback(collector_context, c"Raylib::file_copy".as_ptr(), Raylib_FileCopy);
    register_callback(collector_context, c"raylib::FileCopy".as_ptr(), Raylib_FileCopy);
    register_callback(collector_context, c"raylib::file_copy".as_ptr(), Raylib_FileCopy);
    register_callback(collector_context, c"FileCopy".as_ptr(), Raylib_FileCopy);
    register_callback(collector_context, c"file_copy".as_ptr(), Raylib_FileCopy);
    register_callback(collector_context, c"Raylib::FileMove".as_ptr(), Raylib_FileMove);
    register_callback(collector_context, c"Raylib::file_move".as_ptr(), Raylib_FileMove);
    register_callback(collector_context, c"raylib::FileMove".as_ptr(), Raylib_FileMove);
    register_callback(collector_context, c"raylib::file_move".as_ptr(), Raylib_FileMove);
    register_callback(collector_context, c"FileMove".as_ptr(), Raylib_FileMove);
    register_callback(collector_context, c"file_move".as_ptr(), Raylib_FileMove);
    register_callback(collector_context, c"Raylib::FileTextReplace".as_ptr(), Raylib_FileTextReplace);
    register_callback(collector_context, c"Raylib::file_text_replace".as_ptr(), Raylib_FileTextReplace);
    register_callback(collector_context, c"raylib::FileTextReplace".as_ptr(), Raylib_FileTextReplace);
    register_callback(collector_context, c"raylib::file_text_replace".as_ptr(), Raylib_FileTextReplace);
    register_callback(collector_context, c"FileTextReplace".as_ptr(), Raylib_FileTextReplace);
    register_callback(collector_context, c"file_text_replace".as_ptr(), Raylib_FileTextReplace);
    register_callback(collector_context, c"Raylib::FileTextFindIndex".as_ptr(), Raylib_FileTextFindIndex);
    register_callback(collector_context, c"Raylib::file_text_find_index".as_ptr(), Raylib_FileTextFindIndex);
    register_callback(collector_context, c"raylib::FileTextFindIndex".as_ptr(), Raylib_FileTextFindIndex);
    register_callback(collector_context, c"raylib::file_text_find_index".as_ptr(), Raylib_FileTextFindIndex);
    register_callback(collector_context, c"FileTextFindIndex".as_ptr(), Raylib_FileTextFindIndex);
    register_callback(collector_context, c"file_text_find_index".as_ptr(), Raylib_FileTextFindIndex);
    register_callback(collector_context, c"Raylib::FileExists".as_ptr(), Raylib_FileExists);
    register_callback(collector_context, c"Raylib::file_exists".as_ptr(), Raylib_FileExists);
    register_callback(collector_context, c"raylib::FileExists".as_ptr(), Raylib_FileExists);
    register_callback(collector_context, c"raylib::file_exists".as_ptr(), Raylib_FileExists);
    register_callback(collector_context, c"FileExists".as_ptr(), Raylib_FileExists);
    register_callback(collector_context, c"file_exists".as_ptr(), Raylib_FileExists);
    register_callback(collector_context, c"Raylib::DirectoryExists".as_ptr(), Raylib_DirectoryExists);
    register_callback(collector_context, c"Raylib::directory_exists".as_ptr(), Raylib_DirectoryExists);
    register_callback(collector_context, c"raylib::DirectoryExists".as_ptr(), Raylib_DirectoryExists);
    register_callback(collector_context, c"raylib::directory_exists".as_ptr(), Raylib_DirectoryExists);
    register_callback(collector_context, c"DirectoryExists".as_ptr(), Raylib_DirectoryExists);
    register_callback(collector_context, c"directory_exists".as_ptr(), Raylib_DirectoryExists);
    register_callback(collector_context, c"Raylib::IsFileExtension".as_ptr(), Raylib_IsFileExtension);
    register_callback(collector_context, c"Raylib::is_file_extension".as_ptr(), Raylib_IsFileExtension);
    register_callback(collector_context, c"raylib::IsFileExtension".as_ptr(), Raylib_IsFileExtension);
    register_callback(collector_context, c"raylib::is_file_extension".as_ptr(), Raylib_IsFileExtension);
    register_callback(collector_context, c"IsFileExtension".as_ptr(), Raylib_IsFileExtension);
    register_callback(collector_context, c"is_file_extension".as_ptr(), Raylib_IsFileExtension);
    register_callback(collector_context, c"Raylib::IsFileHidden".as_ptr(), Raylib_IsFileHidden);
    register_callback(collector_context, c"Raylib::is_file_hidden".as_ptr(), Raylib_IsFileHidden);
    register_callback(collector_context, c"raylib::IsFileHidden".as_ptr(), Raylib_IsFileHidden);
    register_callback(collector_context, c"raylib::is_file_hidden".as_ptr(), Raylib_IsFileHidden);
    register_callback(collector_context, c"IsFileHidden".as_ptr(), Raylib_IsFileHidden);
    register_callback(collector_context, c"is_file_hidden".as_ptr(), Raylib_IsFileHidden);
    register_callback(collector_context, c"Raylib::GetFileLength".as_ptr(), Raylib_GetFileLength);
    register_callback(collector_context, c"Raylib::get_file_length".as_ptr(), Raylib_GetFileLength);
    register_callback(collector_context, c"raylib::GetFileLength".as_ptr(), Raylib_GetFileLength);
    register_callback(collector_context, c"raylib::get_file_length".as_ptr(), Raylib_GetFileLength);
    register_callback(collector_context, c"GetFileLength".as_ptr(), Raylib_GetFileLength);
    register_callback(collector_context, c"get_file_length".as_ptr(), Raylib_GetFileLength);
    register_callback(collector_context, c"Raylib::GetFileModTime".as_ptr(), Raylib_GetFileModTime);
    register_callback(collector_context, c"Raylib::get_file_mod_time".as_ptr(), Raylib_GetFileModTime);
    register_callback(collector_context, c"raylib::GetFileModTime".as_ptr(), Raylib_GetFileModTime);
    register_callback(collector_context, c"raylib::get_file_mod_time".as_ptr(), Raylib_GetFileModTime);
    register_callback(collector_context, c"GetFileModTime".as_ptr(), Raylib_GetFileModTime);
    register_callback(collector_context, c"get_file_mod_time".as_ptr(), Raylib_GetFileModTime);
    register_callback(collector_context, c"Raylib::GetFileExtension".as_ptr(), Raylib_GetFileExtension);
    register_callback(collector_context, c"Raylib::get_file_extension".as_ptr(), Raylib_GetFileExtension);
    register_callback(collector_context, c"raylib::GetFileExtension".as_ptr(), Raylib_GetFileExtension);
    register_callback(collector_context, c"raylib::get_file_extension".as_ptr(), Raylib_GetFileExtension);
    register_callback(collector_context, c"GetFileExtension".as_ptr(), Raylib_GetFileExtension);
    register_callback(collector_context, c"get_file_extension".as_ptr(), Raylib_GetFileExtension);
    register_callback(collector_context, c"Raylib::GetFileName".as_ptr(), Raylib_GetFileName);
    register_callback(collector_context, c"Raylib::get_file_name".as_ptr(), Raylib_GetFileName);
    register_callback(collector_context, c"raylib::GetFileName".as_ptr(), Raylib_GetFileName);
    register_callback(collector_context, c"raylib::get_file_name".as_ptr(), Raylib_GetFileName);
    register_callback(collector_context, c"GetFileName".as_ptr(), Raylib_GetFileName);
    register_callback(collector_context, c"get_file_name".as_ptr(), Raylib_GetFileName);
    register_callback(collector_context, c"Raylib::GetFileNameWithoutExt".as_ptr(), Raylib_GetFileNameWithoutExt);
    register_callback(collector_context, c"Raylib::get_file_name_without_ext".as_ptr(), Raylib_GetFileNameWithoutExt);
    register_callback(collector_context, c"raylib::GetFileNameWithoutExt".as_ptr(), Raylib_GetFileNameWithoutExt);
    register_callback(collector_context, c"raylib::get_file_name_without_ext".as_ptr(), Raylib_GetFileNameWithoutExt);
    register_callback(collector_context, c"GetFileNameWithoutExt".as_ptr(), Raylib_GetFileNameWithoutExt);
    register_callback(collector_context, c"get_file_name_without_ext".as_ptr(), Raylib_GetFileNameWithoutExt);
    register_callback(collector_context, c"Raylib::GetDirectoryPath".as_ptr(), Raylib_GetDirectoryPath);
    register_callback(collector_context, c"Raylib::get_directory_path".as_ptr(), Raylib_GetDirectoryPath);
    register_callback(collector_context, c"raylib::GetDirectoryPath".as_ptr(), Raylib_GetDirectoryPath);
    register_callback(collector_context, c"raylib::get_directory_path".as_ptr(), Raylib_GetDirectoryPath);
    register_callback(collector_context, c"GetDirectoryPath".as_ptr(), Raylib_GetDirectoryPath);
    register_callback(collector_context, c"get_directory_path".as_ptr(), Raylib_GetDirectoryPath);
    register_callback(collector_context, c"Raylib::GetPrevDirectoryPath".as_ptr(), Raylib_GetPrevDirectoryPath);
    register_callback(collector_context, c"Raylib::get_prev_directory_path".as_ptr(), Raylib_GetPrevDirectoryPath);
    register_callback(collector_context, c"raylib::GetPrevDirectoryPath".as_ptr(), Raylib_GetPrevDirectoryPath);
    register_callback(collector_context, c"raylib::get_prev_directory_path".as_ptr(), Raylib_GetPrevDirectoryPath);
    register_callback(collector_context, c"GetPrevDirectoryPath".as_ptr(), Raylib_GetPrevDirectoryPath);
    register_callback(collector_context, c"get_prev_directory_path".as_ptr(), Raylib_GetPrevDirectoryPath);
    register_callback(collector_context, c"Raylib::GetWorkingDirectory".as_ptr(), Raylib_GetWorkingDirectory);
    register_callback(collector_context, c"Raylib::get_working_directory".as_ptr(), Raylib_GetWorkingDirectory);
    register_callback(collector_context, c"raylib::GetWorkingDirectory".as_ptr(), Raylib_GetWorkingDirectory);
    register_callback(collector_context, c"raylib::get_working_directory".as_ptr(), Raylib_GetWorkingDirectory);
    register_callback(collector_context, c"GetWorkingDirectory".as_ptr(), Raylib_GetWorkingDirectory);
    register_callback(collector_context, c"get_working_directory".as_ptr(), Raylib_GetWorkingDirectory);
    register_callback(collector_context, c"Raylib::GetApplicationDirectory".as_ptr(), Raylib_GetApplicationDirectory);
    register_callback(collector_context, c"Raylib::get_application_directory".as_ptr(), Raylib_GetApplicationDirectory);
    register_callback(collector_context, c"raylib::GetApplicationDirectory".as_ptr(), Raylib_GetApplicationDirectory);
    register_callback(collector_context, c"raylib::get_application_directory".as_ptr(), Raylib_GetApplicationDirectory);
    register_callback(collector_context, c"GetApplicationDirectory".as_ptr(), Raylib_GetApplicationDirectory);
    register_callback(collector_context, c"get_application_directory".as_ptr(), Raylib_GetApplicationDirectory);
    register_callback(collector_context, c"Raylib::MakeDirectory".as_ptr(), Raylib_MakeDirectory);
    register_callback(collector_context, c"Raylib::make_directory".as_ptr(), Raylib_MakeDirectory);
    register_callback(collector_context, c"raylib::MakeDirectory".as_ptr(), Raylib_MakeDirectory);
    register_callback(collector_context, c"raylib::make_directory".as_ptr(), Raylib_MakeDirectory);
    register_callback(collector_context, c"MakeDirectory".as_ptr(), Raylib_MakeDirectory);
    register_callback(collector_context, c"make_directory".as_ptr(), Raylib_MakeDirectory);
    register_callback(collector_context, c"Raylib::ChangeDirectory".as_ptr(), Raylib_ChangeDirectory);
    register_callback(collector_context, c"Raylib::change_directory".as_ptr(), Raylib_ChangeDirectory);
    register_callback(collector_context, c"raylib::ChangeDirectory".as_ptr(), Raylib_ChangeDirectory);
    register_callback(collector_context, c"raylib::change_directory".as_ptr(), Raylib_ChangeDirectory);
    register_callback(collector_context, c"ChangeDirectory".as_ptr(), Raylib_ChangeDirectory);
    register_callback(collector_context, c"change_directory".as_ptr(), Raylib_ChangeDirectory);
    register_callback(collector_context, c"Raylib::IsPathFile".as_ptr(), Raylib_IsPathFile);
    register_callback(collector_context, c"Raylib::is_path_file".as_ptr(), Raylib_IsPathFile);
    register_callback(collector_context, c"raylib::IsPathFile".as_ptr(), Raylib_IsPathFile);
    register_callback(collector_context, c"raylib::is_path_file".as_ptr(), Raylib_IsPathFile);
    register_callback(collector_context, c"IsPathFile".as_ptr(), Raylib_IsPathFile);
    register_callback(collector_context, c"is_path_file".as_ptr(), Raylib_IsPathFile);
    register_callback(collector_context, c"Raylib::IsPathDirectory".as_ptr(), Raylib_IsPathDirectory);
    register_callback(collector_context, c"Raylib::is_path_directory".as_ptr(), Raylib_IsPathDirectory);
    register_callback(collector_context, c"raylib::IsPathDirectory".as_ptr(), Raylib_IsPathDirectory);
    register_callback(collector_context, c"raylib::is_path_directory".as_ptr(), Raylib_IsPathDirectory);
    register_callback(collector_context, c"IsPathDirectory".as_ptr(), Raylib_IsPathDirectory);
    register_callback(collector_context, c"is_path_directory".as_ptr(), Raylib_IsPathDirectory);
    register_callback(collector_context, c"Raylib::IsPathAbsolute".as_ptr(), Raylib_IsPathAbsolute);
    register_callback(collector_context, c"Raylib::is_path_absolute".as_ptr(), Raylib_IsPathAbsolute);
    register_callback(collector_context, c"raylib::IsPathAbsolute".as_ptr(), Raylib_IsPathAbsolute);
    register_callback(collector_context, c"raylib::is_path_absolute".as_ptr(), Raylib_IsPathAbsolute);
    register_callback(collector_context, c"IsPathAbsolute".as_ptr(), Raylib_IsPathAbsolute);
    register_callback(collector_context, c"is_path_absolute".as_ptr(), Raylib_IsPathAbsolute);
    register_callback(collector_context, c"Raylib::IsFileNameValid".as_ptr(), Raylib_IsFileNameValid);
    register_callback(collector_context, c"Raylib::is_file_name_valid".as_ptr(), Raylib_IsFileNameValid);
    register_callback(collector_context, c"raylib::IsFileNameValid".as_ptr(), Raylib_IsFileNameValid);
    register_callback(collector_context, c"raylib::is_file_name_valid".as_ptr(), Raylib_IsFileNameValid);
    register_callback(collector_context, c"IsFileNameValid".as_ptr(), Raylib_IsFileNameValid);
    register_callback(collector_context, c"is_file_name_valid".as_ptr(), Raylib_IsFileNameValid);
    register_callback(collector_context, c"Raylib::LoadDirectoryFiles".as_ptr(), Raylib_LoadDirectoryFiles);
    register_callback(collector_context, c"Raylib::load_directory_files".as_ptr(), Raylib_LoadDirectoryFiles);
    register_callback(collector_context, c"raylib::LoadDirectoryFiles".as_ptr(), Raylib_LoadDirectoryFiles);
    register_callback(collector_context, c"raylib::load_directory_files".as_ptr(), Raylib_LoadDirectoryFiles);
    register_callback(collector_context, c"LoadDirectoryFiles".as_ptr(), Raylib_LoadDirectoryFiles);
    register_callback(collector_context, c"load_directory_files".as_ptr(), Raylib_LoadDirectoryFiles);
    register_callback(collector_context, c"Raylib::LoadDirectoryFilesEx".as_ptr(), Raylib_LoadDirectoryFilesEx);
    register_callback(collector_context, c"Raylib::load_directory_files_ex".as_ptr(), Raylib_LoadDirectoryFilesEx);
    register_callback(collector_context, c"raylib::LoadDirectoryFilesEx".as_ptr(), Raylib_LoadDirectoryFilesEx);
    register_callback(collector_context, c"raylib::load_directory_files_ex".as_ptr(), Raylib_LoadDirectoryFilesEx);
    register_callback(collector_context, c"LoadDirectoryFilesEx".as_ptr(), Raylib_LoadDirectoryFilesEx);
    register_callback(collector_context, c"load_directory_files_ex".as_ptr(), Raylib_LoadDirectoryFilesEx);
    register_callback(collector_context, c"Raylib::UnloadDirectoryFiles".as_ptr(), Raylib_UnloadDirectoryFiles);
    register_callback(collector_context, c"Raylib::unload_directory_files".as_ptr(), Raylib_UnloadDirectoryFiles);
    register_callback(collector_context, c"raylib::UnloadDirectoryFiles".as_ptr(), Raylib_UnloadDirectoryFiles);
    register_callback(collector_context, c"raylib::unload_directory_files".as_ptr(), Raylib_UnloadDirectoryFiles);
    register_callback(collector_context, c"UnloadDirectoryFiles".as_ptr(), Raylib_UnloadDirectoryFiles);
    register_callback(collector_context, c"unload_directory_files".as_ptr(), Raylib_UnloadDirectoryFiles);
    register_callback(collector_context, c"Raylib::IsFileDropped".as_ptr(), Raylib_IsFileDropped);
    register_callback(collector_context, c"Raylib::is_file_dropped".as_ptr(), Raylib_IsFileDropped);
    register_callback(collector_context, c"raylib::IsFileDropped".as_ptr(), Raylib_IsFileDropped);
    register_callback(collector_context, c"raylib::is_file_dropped".as_ptr(), Raylib_IsFileDropped);
    register_callback(collector_context, c"IsFileDropped".as_ptr(), Raylib_IsFileDropped);
    register_callback(collector_context, c"is_file_dropped".as_ptr(), Raylib_IsFileDropped);
    register_callback(collector_context, c"Raylib::LoadDroppedFiles".as_ptr(), Raylib_LoadDroppedFiles);
    register_callback(collector_context, c"Raylib::load_dropped_files".as_ptr(), Raylib_LoadDroppedFiles);
    register_callback(collector_context, c"raylib::LoadDroppedFiles".as_ptr(), Raylib_LoadDroppedFiles);
    register_callback(collector_context, c"raylib::load_dropped_files".as_ptr(), Raylib_LoadDroppedFiles);
    register_callback(collector_context, c"LoadDroppedFiles".as_ptr(), Raylib_LoadDroppedFiles);
    register_callback(collector_context, c"load_dropped_files".as_ptr(), Raylib_LoadDroppedFiles);
    register_callback(collector_context, c"Raylib::UnloadDroppedFiles".as_ptr(), Raylib_UnloadDroppedFiles);
    register_callback(collector_context, c"Raylib::unload_dropped_files".as_ptr(), Raylib_UnloadDroppedFiles);
    register_callback(collector_context, c"raylib::UnloadDroppedFiles".as_ptr(), Raylib_UnloadDroppedFiles);
    register_callback(collector_context, c"raylib::unload_dropped_files".as_ptr(), Raylib_UnloadDroppedFiles);
    register_callback(collector_context, c"UnloadDroppedFiles".as_ptr(), Raylib_UnloadDroppedFiles);
    register_callback(collector_context, c"unload_dropped_files".as_ptr(), Raylib_UnloadDroppedFiles);
    register_callback(collector_context, c"Raylib::GetDirectoryFileCount".as_ptr(), Raylib_GetDirectoryFileCount);
    register_callback(collector_context, c"Raylib::get_directory_file_count".as_ptr(), Raylib_GetDirectoryFileCount);
    register_callback(collector_context, c"raylib::GetDirectoryFileCount".as_ptr(), Raylib_GetDirectoryFileCount);
    register_callback(collector_context, c"raylib::get_directory_file_count".as_ptr(), Raylib_GetDirectoryFileCount);
    register_callback(collector_context, c"GetDirectoryFileCount".as_ptr(), Raylib_GetDirectoryFileCount);
    register_callback(collector_context, c"get_directory_file_count".as_ptr(), Raylib_GetDirectoryFileCount);
    register_callback(collector_context, c"Raylib::GetDirectoryFileCountEx".as_ptr(), Raylib_GetDirectoryFileCountEx);
    register_callback(collector_context, c"Raylib::get_directory_file_count_ex".as_ptr(), Raylib_GetDirectoryFileCountEx);
    register_callback(collector_context, c"raylib::GetDirectoryFileCountEx".as_ptr(), Raylib_GetDirectoryFileCountEx);
    register_callback(collector_context, c"raylib::get_directory_file_count_ex".as_ptr(), Raylib_GetDirectoryFileCountEx);
    register_callback(collector_context, c"GetDirectoryFileCountEx".as_ptr(), Raylib_GetDirectoryFileCountEx);
    register_callback(collector_context, c"get_directory_file_count_ex".as_ptr(), Raylib_GetDirectoryFileCountEx);
    register_callback(collector_context, c"Raylib::CompressData".as_ptr(), Raylib_CompressData);
    register_callback(collector_context, c"Raylib::compress_data".as_ptr(), Raylib_CompressData);
    register_callback(collector_context, c"raylib::CompressData".as_ptr(), Raylib_CompressData);
    register_callback(collector_context, c"raylib::compress_data".as_ptr(), Raylib_CompressData);
    register_callback(collector_context, c"CompressData".as_ptr(), Raylib_CompressData);
    register_callback(collector_context, c"compress_data".as_ptr(), Raylib_CompressData);
    register_callback(collector_context, c"Raylib::DecompressData".as_ptr(), Raylib_DecompressData);
    register_callback(collector_context, c"Raylib::decompress_data".as_ptr(), Raylib_DecompressData);
    register_callback(collector_context, c"raylib::DecompressData".as_ptr(), Raylib_DecompressData);
    register_callback(collector_context, c"raylib::decompress_data".as_ptr(), Raylib_DecompressData);
    register_callback(collector_context, c"DecompressData".as_ptr(), Raylib_DecompressData);
    register_callback(collector_context, c"decompress_data".as_ptr(), Raylib_DecompressData);
    register_callback(collector_context, c"Raylib::EncodeDataBase64".as_ptr(), Raylib_EncodeDataBase64);
    register_callback(collector_context, c"Raylib::encode_data_base64".as_ptr(), Raylib_EncodeDataBase64);
    register_callback(collector_context, c"raylib::EncodeDataBase64".as_ptr(), Raylib_EncodeDataBase64);
    register_callback(collector_context, c"raylib::encode_data_base64".as_ptr(), Raylib_EncodeDataBase64);
    register_callback(collector_context, c"EncodeDataBase64".as_ptr(), Raylib_EncodeDataBase64);
    register_callback(collector_context, c"encode_data_base64".as_ptr(), Raylib_EncodeDataBase64);
    register_callback(collector_context, c"Raylib::DecodeDataBase64".as_ptr(), Raylib_DecodeDataBase64);
    register_callback(collector_context, c"Raylib::decode_data_base64".as_ptr(), Raylib_DecodeDataBase64);
    register_callback(collector_context, c"raylib::DecodeDataBase64".as_ptr(), Raylib_DecodeDataBase64);
    register_callback(collector_context, c"raylib::decode_data_base64".as_ptr(), Raylib_DecodeDataBase64);
    register_callback(collector_context, c"DecodeDataBase64".as_ptr(), Raylib_DecodeDataBase64);
    register_callback(collector_context, c"decode_data_base64".as_ptr(), Raylib_DecodeDataBase64);
    register_callback(collector_context, c"Raylib::ComputeCRC32".as_ptr(), Raylib_ComputeCRC32);
    register_callback(collector_context, c"Raylib::compute_crc32".as_ptr(), Raylib_ComputeCRC32);
    register_callback(collector_context, c"raylib::ComputeCRC32".as_ptr(), Raylib_ComputeCRC32);
    register_callback(collector_context, c"raylib::compute_crc32".as_ptr(), Raylib_ComputeCRC32);
    register_callback(collector_context, c"ComputeCRC32".as_ptr(), Raylib_ComputeCRC32);
    register_callback(collector_context, c"compute_crc32".as_ptr(), Raylib_ComputeCRC32);
    register_callback(collector_context, c"Raylib::ComputeMD5".as_ptr(), Raylib_ComputeMD5);
    register_callback(collector_context, c"Raylib::compute_md5".as_ptr(), Raylib_ComputeMD5);
    register_callback(collector_context, c"raylib::ComputeMD5".as_ptr(), Raylib_ComputeMD5);
    register_callback(collector_context, c"raylib::compute_md5".as_ptr(), Raylib_ComputeMD5);
    register_callback(collector_context, c"ComputeMD5".as_ptr(), Raylib_ComputeMD5);
    register_callback(collector_context, c"compute_md5".as_ptr(), Raylib_ComputeMD5);
    register_callback(collector_context, c"Raylib::ComputeSHA1".as_ptr(), Raylib_ComputeSHA1);
    register_callback(collector_context, c"Raylib::compute_sha1".as_ptr(), Raylib_ComputeSHA1);
    register_callback(collector_context, c"raylib::ComputeSHA1".as_ptr(), Raylib_ComputeSHA1);
    register_callback(collector_context, c"raylib::compute_sha1".as_ptr(), Raylib_ComputeSHA1);
    register_callback(collector_context, c"ComputeSHA1".as_ptr(), Raylib_ComputeSHA1);
    register_callback(collector_context, c"compute_sha1".as_ptr(), Raylib_ComputeSHA1);
    register_callback(collector_context, c"Raylib::ComputeSHA256".as_ptr(), Raylib_ComputeSHA256);
    register_callback(collector_context, c"Raylib::compute_sha256".as_ptr(), Raylib_ComputeSHA256);
    register_callback(collector_context, c"raylib::ComputeSHA256".as_ptr(), Raylib_ComputeSHA256);
    register_callback(collector_context, c"raylib::compute_sha256".as_ptr(), Raylib_ComputeSHA256);
    register_callback(collector_context, c"ComputeSHA256".as_ptr(), Raylib_ComputeSHA256);
    register_callback(collector_context, c"compute_sha256".as_ptr(), Raylib_ComputeSHA256);
    register_callback(collector_context, c"Raylib::LoadAutomationEventList".as_ptr(), Raylib_LoadAutomationEventList);
    register_callback(collector_context, c"Raylib::load_automation_event_list".as_ptr(), Raylib_LoadAutomationEventList);
    register_callback(collector_context, c"raylib::LoadAutomationEventList".as_ptr(), Raylib_LoadAutomationEventList);
    register_callback(collector_context, c"raylib::load_automation_event_list".as_ptr(), Raylib_LoadAutomationEventList);
    register_callback(collector_context, c"LoadAutomationEventList".as_ptr(), Raylib_LoadAutomationEventList);
    register_callback(collector_context, c"load_automation_event_list".as_ptr(), Raylib_LoadAutomationEventList);
    register_callback(collector_context, c"Raylib::UnloadAutomationEventList".as_ptr(), Raylib_UnloadAutomationEventList);
    register_callback(collector_context, c"Raylib::unload_automation_event_list".as_ptr(), Raylib_UnloadAutomationEventList);
    register_callback(collector_context, c"raylib::UnloadAutomationEventList".as_ptr(), Raylib_UnloadAutomationEventList);
    register_callback(collector_context, c"raylib::unload_automation_event_list".as_ptr(), Raylib_UnloadAutomationEventList);
    register_callback(collector_context, c"UnloadAutomationEventList".as_ptr(), Raylib_UnloadAutomationEventList);
    register_callback(collector_context, c"unload_automation_event_list".as_ptr(), Raylib_UnloadAutomationEventList);
    register_callback(collector_context, c"Raylib::ExportAutomationEventList".as_ptr(), Raylib_ExportAutomationEventList);
    register_callback(collector_context, c"Raylib::export_automation_event_list".as_ptr(), Raylib_ExportAutomationEventList);
    register_callback(collector_context, c"raylib::ExportAutomationEventList".as_ptr(), Raylib_ExportAutomationEventList);
    register_callback(collector_context, c"raylib::export_automation_event_list".as_ptr(), Raylib_ExportAutomationEventList);
    register_callback(collector_context, c"ExportAutomationEventList".as_ptr(), Raylib_ExportAutomationEventList);
    register_callback(collector_context, c"export_automation_event_list".as_ptr(), Raylib_ExportAutomationEventList);
    register_callback(collector_context, c"Raylib::SetAutomationEventList".as_ptr(), Raylib_SetAutomationEventList);
    register_callback(collector_context, c"Raylib::set_automation_event_list".as_ptr(), Raylib_SetAutomationEventList);
    register_callback(collector_context, c"raylib::SetAutomationEventList".as_ptr(), Raylib_SetAutomationEventList);
    register_callback(collector_context, c"raylib::set_automation_event_list".as_ptr(), Raylib_SetAutomationEventList);
    register_callback(collector_context, c"SetAutomationEventList".as_ptr(), Raylib_SetAutomationEventList);
    register_callback(collector_context, c"set_automation_event_list".as_ptr(), Raylib_SetAutomationEventList);
    register_callback(collector_context, c"Raylib::SetAutomationEventBaseFrame".as_ptr(), Raylib_SetAutomationEventBaseFrame);
    register_callback(collector_context, c"Raylib::set_automation_event_base_frame".as_ptr(), Raylib_SetAutomationEventBaseFrame);
    register_callback(collector_context, c"raylib::SetAutomationEventBaseFrame".as_ptr(), Raylib_SetAutomationEventBaseFrame);
    register_callback(collector_context, c"raylib::set_automation_event_base_frame".as_ptr(), Raylib_SetAutomationEventBaseFrame);
    register_callback(collector_context, c"SetAutomationEventBaseFrame".as_ptr(), Raylib_SetAutomationEventBaseFrame);
    register_callback(collector_context, c"set_automation_event_base_frame".as_ptr(), Raylib_SetAutomationEventBaseFrame);
    register_callback(collector_context, c"Raylib::StartAutomationEventRecording".as_ptr(), Raylib_StartAutomationEventRecording);
    register_callback(collector_context, c"Raylib::start_automation_event_recording".as_ptr(), Raylib_StartAutomationEventRecording);
    register_callback(collector_context, c"raylib::StartAutomationEventRecording".as_ptr(), Raylib_StartAutomationEventRecording);
    register_callback(collector_context, c"raylib::start_automation_event_recording".as_ptr(), Raylib_StartAutomationEventRecording);
    register_callback(collector_context, c"StartAutomationEventRecording".as_ptr(), Raylib_StartAutomationEventRecording);
    register_callback(collector_context, c"start_automation_event_recording".as_ptr(), Raylib_StartAutomationEventRecording);
    register_callback(collector_context, c"Raylib::StopAutomationEventRecording".as_ptr(), Raylib_StopAutomationEventRecording);
    register_callback(collector_context, c"Raylib::stop_automation_event_recording".as_ptr(), Raylib_StopAutomationEventRecording);
    register_callback(collector_context, c"raylib::StopAutomationEventRecording".as_ptr(), Raylib_StopAutomationEventRecording);
    register_callback(collector_context, c"raylib::stop_automation_event_recording".as_ptr(), Raylib_StopAutomationEventRecording);
    register_callback(collector_context, c"StopAutomationEventRecording".as_ptr(), Raylib_StopAutomationEventRecording);
    register_callback(collector_context, c"stop_automation_event_recording".as_ptr(), Raylib_StopAutomationEventRecording);
    register_callback(collector_context, c"Raylib::PlayAutomationEvent".as_ptr(), Raylib_PlayAutomationEvent);
    register_callback(collector_context, c"Raylib::play_automation_event".as_ptr(), Raylib_PlayAutomationEvent);
    register_callback(collector_context, c"raylib::PlayAutomationEvent".as_ptr(), Raylib_PlayAutomationEvent);
    register_callback(collector_context, c"raylib::play_automation_event".as_ptr(), Raylib_PlayAutomationEvent);
    register_callback(collector_context, c"PlayAutomationEvent".as_ptr(), Raylib_PlayAutomationEvent);
    register_callback(collector_context, c"play_automation_event".as_ptr(), Raylib_PlayAutomationEvent);
    register_callback(collector_context, c"Raylib::IsKeyPressed".as_ptr(), Raylib_IsKeyPressed);
    register_callback(collector_context, c"Raylib::is_key_pressed".as_ptr(), Raylib_IsKeyPressed);
    register_callback(collector_context, c"raylib::IsKeyPressed".as_ptr(), Raylib_IsKeyPressed);
    register_callback(collector_context, c"raylib::is_key_pressed".as_ptr(), Raylib_IsKeyPressed);
    register_callback(collector_context, c"IsKeyPressed".as_ptr(), Raylib_IsKeyPressed);
    register_callback(collector_context, c"is_key_pressed".as_ptr(), Raylib_IsKeyPressed);
    register_callback(collector_context, c"Raylib::IsKeyPressedRepeat".as_ptr(), Raylib_IsKeyPressedRepeat);
    register_callback(collector_context, c"Raylib::is_key_pressed_repeat".as_ptr(), Raylib_IsKeyPressedRepeat);
    register_callback(collector_context, c"raylib::IsKeyPressedRepeat".as_ptr(), Raylib_IsKeyPressedRepeat);
    register_callback(collector_context, c"raylib::is_key_pressed_repeat".as_ptr(), Raylib_IsKeyPressedRepeat);
    register_callback(collector_context, c"IsKeyPressedRepeat".as_ptr(), Raylib_IsKeyPressedRepeat);
    register_callback(collector_context, c"is_key_pressed_repeat".as_ptr(), Raylib_IsKeyPressedRepeat);
    register_callback(collector_context, c"Raylib::IsKeyDown".as_ptr(), Raylib_IsKeyDown);
    register_callback(collector_context, c"Raylib::is_key_down".as_ptr(), Raylib_IsKeyDown);
    register_callback(collector_context, c"raylib::IsKeyDown".as_ptr(), Raylib_IsKeyDown);
    register_callback(collector_context, c"raylib::is_key_down".as_ptr(), Raylib_IsKeyDown);
    register_callback(collector_context, c"IsKeyDown".as_ptr(), Raylib_IsKeyDown);
    register_callback(collector_context, c"is_key_down".as_ptr(), Raylib_IsKeyDown);
    register_callback(collector_context, c"Raylib::IsKeyReleased".as_ptr(), Raylib_IsKeyReleased);
    register_callback(collector_context, c"Raylib::is_key_released".as_ptr(), Raylib_IsKeyReleased);
    register_callback(collector_context, c"raylib::IsKeyReleased".as_ptr(), Raylib_IsKeyReleased);
    register_callback(collector_context, c"raylib::is_key_released".as_ptr(), Raylib_IsKeyReleased);
    register_callback(collector_context, c"IsKeyReleased".as_ptr(), Raylib_IsKeyReleased);
    register_callback(collector_context, c"is_key_released".as_ptr(), Raylib_IsKeyReleased);
    register_callback(collector_context, c"Raylib::IsKeyUp".as_ptr(), Raylib_IsKeyUp);
    register_callback(collector_context, c"Raylib::is_key_up".as_ptr(), Raylib_IsKeyUp);
    register_callback(collector_context, c"raylib::IsKeyUp".as_ptr(), Raylib_IsKeyUp);
    register_callback(collector_context, c"raylib::is_key_up".as_ptr(), Raylib_IsKeyUp);
    register_callback(collector_context, c"IsKeyUp".as_ptr(), Raylib_IsKeyUp);
    register_callback(collector_context, c"is_key_up".as_ptr(), Raylib_IsKeyUp);
    register_callback(collector_context, c"Raylib::GetKeyPressed".as_ptr(), Raylib_GetKeyPressed);
    register_callback(collector_context, c"Raylib::get_key_pressed".as_ptr(), Raylib_GetKeyPressed);
    register_callback(collector_context, c"raylib::GetKeyPressed".as_ptr(), Raylib_GetKeyPressed);
    register_callback(collector_context, c"raylib::get_key_pressed".as_ptr(), Raylib_GetKeyPressed);
    register_callback(collector_context, c"GetKeyPressed".as_ptr(), Raylib_GetKeyPressed);
    register_callback(collector_context, c"get_key_pressed".as_ptr(), Raylib_GetKeyPressed);
    register_callback(collector_context, c"Raylib::GetCharPressed".as_ptr(), Raylib_GetCharPressed);
    register_callback(collector_context, c"Raylib::get_char_pressed".as_ptr(), Raylib_GetCharPressed);
    register_callback(collector_context, c"raylib::GetCharPressed".as_ptr(), Raylib_GetCharPressed);
    register_callback(collector_context, c"raylib::get_char_pressed".as_ptr(), Raylib_GetCharPressed);
    register_callback(collector_context, c"GetCharPressed".as_ptr(), Raylib_GetCharPressed);
    register_callback(collector_context, c"get_char_pressed".as_ptr(), Raylib_GetCharPressed);
    register_callback(collector_context, c"Raylib::GetKeyName".as_ptr(), Raylib_GetKeyName);
    register_callback(collector_context, c"Raylib::get_key_name".as_ptr(), Raylib_GetKeyName);
    register_callback(collector_context, c"raylib::GetKeyName".as_ptr(), Raylib_GetKeyName);
    register_callback(collector_context, c"raylib::get_key_name".as_ptr(), Raylib_GetKeyName);
    register_callback(collector_context, c"GetKeyName".as_ptr(), Raylib_GetKeyName);
    register_callback(collector_context, c"get_key_name".as_ptr(), Raylib_GetKeyName);
    register_callback(collector_context, c"Raylib::SetExitKey".as_ptr(), Raylib_SetExitKey);
    register_callback(collector_context, c"Raylib::set_exit_key".as_ptr(), Raylib_SetExitKey);
    register_callback(collector_context, c"raylib::SetExitKey".as_ptr(), Raylib_SetExitKey);
    register_callback(collector_context, c"raylib::set_exit_key".as_ptr(), Raylib_SetExitKey);
    register_callback(collector_context, c"SetExitKey".as_ptr(), Raylib_SetExitKey);
    register_callback(collector_context, c"set_exit_key".as_ptr(), Raylib_SetExitKey);
    register_callback(collector_context, c"Raylib::IsGamepadAvailable".as_ptr(), Raylib_IsGamepadAvailable);
    register_callback(collector_context, c"Raylib::is_gamepad_available".as_ptr(), Raylib_IsGamepadAvailable);
    register_callback(collector_context, c"raylib::IsGamepadAvailable".as_ptr(), Raylib_IsGamepadAvailable);
    register_callback(collector_context, c"raylib::is_gamepad_available".as_ptr(), Raylib_IsGamepadAvailable);
    register_callback(collector_context, c"IsGamepadAvailable".as_ptr(), Raylib_IsGamepadAvailable);
    register_callback(collector_context, c"is_gamepad_available".as_ptr(), Raylib_IsGamepadAvailable);
    register_callback(collector_context, c"Raylib::GetGamepadName".as_ptr(), Raylib_GetGamepadName);
    register_callback(collector_context, c"Raylib::get_gamepad_name".as_ptr(), Raylib_GetGamepadName);
    register_callback(collector_context, c"raylib::GetGamepadName".as_ptr(), Raylib_GetGamepadName);
    register_callback(collector_context, c"raylib::get_gamepad_name".as_ptr(), Raylib_GetGamepadName);
    register_callback(collector_context, c"GetGamepadName".as_ptr(), Raylib_GetGamepadName);
    register_callback(collector_context, c"get_gamepad_name".as_ptr(), Raylib_GetGamepadName);
    register_callback(collector_context, c"Raylib::IsGamepadButtonPressed".as_ptr(), Raylib_IsGamepadButtonPressed);
    register_callback(collector_context, c"Raylib::is_gamepad_button_pressed".as_ptr(), Raylib_IsGamepadButtonPressed);
    register_callback(collector_context, c"raylib::IsGamepadButtonPressed".as_ptr(), Raylib_IsGamepadButtonPressed);
    register_callback(collector_context, c"raylib::is_gamepad_button_pressed".as_ptr(), Raylib_IsGamepadButtonPressed);
    register_callback(collector_context, c"IsGamepadButtonPressed".as_ptr(), Raylib_IsGamepadButtonPressed);
    register_callback(collector_context, c"is_gamepad_button_pressed".as_ptr(), Raylib_IsGamepadButtonPressed);
    register_callback(collector_context, c"Raylib::IsGamepadButtonDown".as_ptr(), Raylib_IsGamepadButtonDown);
    register_callback(collector_context, c"Raylib::is_gamepad_button_down".as_ptr(), Raylib_IsGamepadButtonDown);
    register_callback(collector_context, c"raylib::IsGamepadButtonDown".as_ptr(), Raylib_IsGamepadButtonDown);
    register_callback(collector_context, c"raylib::is_gamepad_button_down".as_ptr(), Raylib_IsGamepadButtonDown);
    register_callback(collector_context, c"IsGamepadButtonDown".as_ptr(), Raylib_IsGamepadButtonDown);
    register_callback(collector_context, c"is_gamepad_button_down".as_ptr(), Raylib_IsGamepadButtonDown);
    register_callback(collector_context, c"Raylib::IsGamepadButtonReleased".as_ptr(), Raylib_IsGamepadButtonReleased);
    register_callback(collector_context, c"Raylib::is_gamepad_button_released".as_ptr(), Raylib_IsGamepadButtonReleased);
    register_callback(collector_context, c"raylib::IsGamepadButtonReleased".as_ptr(), Raylib_IsGamepadButtonReleased);
    register_callback(collector_context, c"raylib::is_gamepad_button_released".as_ptr(), Raylib_IsGamepadButtonReleased);
    register_callback(collector_context, c"IsGamepadButtonReleased".as_ptr(), Raylib_IsGamepadButtonReleased);
    register_callback(collector_context, c"is_gamepad_button_released".as_ptr(), Raylib_IsGamepadButtonReleased);
    register_callback(collector_context, c"Raylib::IsGamepadButtonUp".as_ptr(), Raylib_IsGamepadButtonUp);
    register_callback(collector_context, c"Raylib::is_gamepad_button_up".as_ptr(), Raylib_IsGamepadButtonUp);
    register_callback(collector_context, c"raylib::IsGamepadButtonUp".as_ptr(), Raylib_IsGamepadButtonUp);
    register_callback(collector_context, c"raylib::is_gamepad_button_up".as_ptr(), Raylib_IsGamepadButtonUp);
    register_callback(collector_context, c"IsGamepadButtonUp".as_ptr(), Raylib_IsGamepadButtonUp);
    register_callback(collector_context, c"is_gamepad_button_up".as_ptr(), Raylib_IsGamepadButtonUp);
    register_callback(collector_context, c"Raylib::GetGamepadButtonPressed".as_ptr(), Raylib_GetGamepadButtonPressed);
    register_callback(collector_context, c"Raylib::get_gamepad_button_pressed".as_ptr(), Raylib_GetGamepadButtonPressed);
    register_callback(collector_context, c"raylib::GetGamepadButtonPressed".as_ptr(), Raylib_GetGamepadButtonPressed);
    register_callback(collector_context, c"raylib::get_gamepad_button_pressed".as_ptr(), Raylib_GetGamepadButtonPressed);
    register_callback(collector_context, c"GetGamepadButtonPressed".as_ptr(), Raylib_GetGamepadButtonPressed);
    register_callback(collector_context, c"get_gamepad_button_pressed".as_ptr(), Raylib_GetGamepadButtonPressed);
    register_callback(collector_context, c"Raylib::GetGamepadAxisCount".as_ptr(), Raylib_GetGamepadAxisCount);
    register_callback(collector_context, c"Raylib::get_gamepad_axis_count".as_ptr(), Raylib_GetGamepadAxisCount);
    register_callback(collector_context, c"raylib::GetGamepadAxisCount".as_ptr(), Raylib_GetGamepadAxisCount);
    register_callback(collector_context, c"raylib::get_gamepad_axis_count".as_ptr(), Raylib_GetGamepadAxisCount);
    register_callback(collector_context, c"GetGamepadAxisCount".as_ptr(), Raylib_GetGamepadAxisCount);
    register_callback(collector_context, c"get_gamepad_axis_count".as_ptr(), Raylib_GetGamepadAxisCount);
    register_callback(collector_context, c"Raylib::GetGamepadAxisMovement".as_ptr(), Raylib_GetGamepadAxisMovement);
    register_callback(collector_context, c"Raylib::get_gamepad_axis_movement".as_ptr(), Raylib_GetGamepadAxisMovement);
    register_callback(collector_context, c"raylib::GetGamepadAxisMovement".as_ptr(), Raylib_GetGamepadAxisMovement);
    register_callback(collector_context, c"raylib::get_gamepad_axis_movement".as_ptr(), Raylib_GetGamepadAxisMovement);
    register_callback(collector_context, c"GetGamepadAxisMovement".as_ptr(), Raylib_GetGamepadAxisMovement);
    register_callback(collector_context, c"get_gamepad_axis_movement".as_ptr(), Raylib_GetGamepadAxisMovement);
    register_callback(collector_context, c"Raylib::SetGamepadMappings".as_ptr(), Raylib_SetGamepadMappings);
    register_callback(collector_context, c"Raylib::set_gamepad_mappings".as_ptr(), Raylib_SetGamepadMappings);
    register_callback(collector_context, c"raylib::SetGamepadMappings".as_ptr(), Raylib_SetGamepadMappings);
    register_callback(collector_context, c"raylib::set_gamepad_mappings".as_ptr(), Raylib_SetGamepadMappings);
    register_callback(collector_context, c"SetGamepadMappings".as_ptr(), Raylib_SetGamepadMappings);
    register_callback(collector_context, c"set_gamepad_mappings".as_ptr(), Raylib_SetGamepadMappings);
    register_callback(collector_context, c"Raylib::SetGamepadVibration".as_ptr(), Raylib_SetGamepadVibration);
    register_callback(collector_context, c"Raylib::set_gamepad_vibration".as_ptr(), Raylib_SetGamepadVibration);
    register_callback(collector_context, c"raylib::SetGamepadVibration".as_ptr(), Raylib_SetGamepadVibration);
    register_callback(collector_context, c"raylib::set_gamepad_vibration".as_ptr(), Raylib_SetGamepadVibration);
    register_callback(collector_context, c"SetGamepadVibration".as_ptr(), Raylib_SetGamepadVibration);
    register_callback(collector_context, c"set_gamepad_vibration".as_ptr(), Raylib_SetGamepadVibration);
    register_callback(collector_context, c"Raylib::IsMouseButtonPressed".as_ptr(), Raylib_IsMouseButtonPressed);
    register_callback(collector_context, c"Raylib::is_mouse_button_pressed".as_ptr(), Raylib_IsMouseButtonPressed);
    register_callback(collector_context, c"raylib::IsMouseButtonPressed".as_ptr(), Raylib_IsMouseButtonPressed);
    register_callback(collector_context, c"raylib::is_mouse_button_pressed".as_ptr(), Raylib_IsMouseButtonPressed);
    register_callback(collector_context, c"IsMouseButtonPressed".as_ptr(), Raylib_IsMouseButtonPressed);
    register_callback(collector_context, c"is_mouse_button_pressed".as_ptr(), Raylib_IsMouseButtonPressed);
    register_callback(collector_context, c"Raylib::IsMouseButtonDown".as_ptr(), Raylib_IsMouseButtonDown);
    register_callback(collector_context, c"Raylib::is_mouse_button_down".as_ptr(), Raylib_IsMouseButtonDown);
    register_callback(collector_context, c"raylib::IsMouseButtonDown".as_ptr(), Raylib_IsMouseButtonDown);
    register_callback(collector_context, c"raylib::is_mouse_button_down".as_ptr(), Raylib_IsMouseButtonDown);
    register_callback(collector_context, c"IsMouseButtonDown".as_ptr(), Raylib_IsMouseButtonDown);
    register_callback(collector_context, c"is_mouse_button_down".as_ptr(), Raylib_IsMouseButtonDown);
    register_callback(collector_context, c"Raylib::IsMouseButtonReleased".as_ptr(), Raylib_IsMouseButtonReleased);
    register_callback(collector_context, c"Raylib::is_mouse_button_released".as_ptr(), Raylib_IsMouseButtonReleased);
    register_callback(collector_context, c"raylib::IsMouseButtonReleased".as_ptr(), Raylib_IsMouseButtonReleased);
    register_callback(collector_context, c"raylib::is_mouse_button_released".as_ptr(), Raylib_IsMouseButtonReleased);
    register_callback(collector_context, c"IsMouseButtonReleased".as_ptr(), Raylib_IsMouseButtonReleased);
    register_callback(collector_context, c"is_mouse_button_released".as_ptr(), Raylib_IsMouseButtonReleased);
    register_callback(collector_context, c"Raylib::IsMouseButtonUp".as_ptr(), Raylib_IsMouseButtonUp);
    register_callback(collector_context, c"Raylib::is_mouse_button_up".as_ptr(), Raylib_IsMouseButtonUp);
    register_callback(collector_context, c"raylib::IsMouseButtonUp".as_ptr(), Raylib_IsMouseButtonUp);
    register_callback(collector_context, c"raylib::is_mouse_button_up".as_ptr(), Raylib_IsMouseButtonUp);
    register_callback(collector_context, c"IsMouseButtonUp".as_ptr(), Raylib_IsMouseButtonUp);
    register_callback(collector_context, c"is_mouse_button_up".as_ptr(), Raylib_IsMouseButtonUp);
    register_callback(collector_context, c"Raylib::GetMouseX".as_ptr(), Raylib_GetMouseX);
    register_callback(collector_context, c"Raylib::get_mouse_x".as_ptr(), Raylib_GetMouseX);
    register_callback(collector_context, c"raylib::GetMouseX".as_ptr(), Raylib_GetMouseX);
    register_callback(collector_context, c"raylib::get_mouse_x".as_ptr(), Raylib_GetMouseX);
    register_callback(collector_context, c"GetMouseX".as_ptr(), Raylib_GetMouseX);
    register_callback(collector_context, c"get_mouse_x".as_ptr(), Raylib_GetMouseX);
    register_callback(collector_context, c"Raylib::GetMouseY".as_ptr(), Raylib_GetMouseY);
    register_callback(collector_context, c"Raylib::get_mouse_y".as_ptr(), Raylib_GetMouseY);
    register_callback(collector_context, c"raylib::GetMouseY".as_ptr(), Raylib_GetMouseY);
    register_callback(collector_context, c"raylib::get_mouse_y".as_ptr(), Raylib_GetMouseY);
    register_callback(collector_context, c"GetMouseY".as_ptr(), Raylib_GetMouseY);
    register_callback(collector_context, c"get_mouse_y".as_ptr(), Raylib_GetMouseY);
    register_callback(collector_context, c"Raylib::GetMousePosition".as_ptr(), Raylib_GetMousePosition);
    register_callback(collector_context, c"Raylib::get_mouse_position".as_ptr(), Raylib_GetMousePosition);
    register_callback(collector_context, c"raylib::GetMousePosition".as_ptr(), Raylib_GetMousePosition);
    register_callback(collector_context, c"raylib::get_mouse_position".as_ptr(), Raylib_GetMousePosition);
    register_callback(collector_context, c"GetMousePosition".as_ptr(), Raylib_GetMousePosition);
    register_callback(collector_context, c"get_mouse_position".as_ptr(), Raylib_GetMousePosition);
    register_callback(collector_context, c"Raylib::GetMouseDelta".as_ptr(), Raylib_GetMouseDelta);
    register_callback(collector_context, c"Raylib::get_mouse_delta".as_ptr(), Raylib_GetMouseDelta);
    register_callback(collector_context, c"raylib::GetMouseDelta".as_ptr(), Raylib_GetMouseDelta);
    register_callback(collector_context, c"raylib::get_mouse_delta".as_ptr(), Raylib_GetMouseDelta);
    register_callback(collector_context, c"GetMouseDelta".as_ptr(), Raylib_GetMouseDelta);
    register_callback(collector_context, c"get_mouse_delta".as_ptr(), Raylib_GetMouseDelta);
    register_callback(collector_context, c"Raylib::SetMousePosition".as_ptr(), Raylib_SetMousePosition);
    register_callback(collector_context, c"Raylib::set_mouse_position".as_ptr(), Raylib_SetMousePosition);
    register_callback(collector_context, c"raylib::SetMousePosition".as_ptr(), Raylib_SetMousePosition);
    register_callback(collector_context, c"raylib::set_mouse_position".as_ptr(), Raylib_SetMousePosition);
    register_callback(collector_context, c"SetMousePosition".as_ptr(), Raylib_SetMousePosition);
    register_callback(collector_context, c"set_mouse_position".as_ptr(), Raylib_SetMousePosition);
    register_callback(collector_context, c"Raylib::SetMouseOffset".as_ptr(), Raylib_SetMouseOffset);
    register_callback(collector_context, c"Raylib::set_mouse_offset".as_ptr(), Raylib_SetMouseOffset);
    register_callback(collector_context, c"raylib::SetMouseOffset".as_ptr(), Raylib_SetMouseOffset);
    register_callback(collector_context, c"raylib::set_mouse_offset".as_ptr(), Raylib_SetMouseOffset);
    register_callback(collector_context, c"SetMouseOffset".as_ptr(), Raylib_SetMouseOffset);
    register_callback(collector_context, c"set_mouse_offset".as_ptr(), Raylib_SetMouseOffset);
    register_callback(collector_context, c"Raylib::SetMouseScale".as_ptr(), Raylib_SetMouseScale);
    register_callback(collector_context, c"Raylib::set_mouse_scale".as_ptr(), Raylib_SetMouseScale);
    register_callback(collector_context, c"raylib::SetMouseScale".as_ptr(), Raylib_SetMouseScale);
    register_callback(collector_context, c"raylib::set_mouse_scale".as_ptr(), Raylib_SetMouseScale);
    register_callback(collector_context, c"SetMouseScale".as_ptr(), Raylib_SetMouseScale);
    register_callback(collector_context, c"set_mouse_scale".as_ptr(), Raylib_SetMouseScale);
    register_callback(collector_context, c"Raylib::GetMouseWheelMove".as_ptr(), Raylib_GetMouseWheelMove);
    register_callback(collector_context, c"Raylib::get_mouse_wheel_move".as_ptr(), Raylib_GetMouseWheelMove);
    register_callback(collector_context, c"raylib::GetMouseWheelMove".as_ptr(), Raylib_GetMouseWheelMove);
    register_callback(collector_context, c"raylib::get_mouse_wheel_move".as_ptr(), Raylib_GetMouseWheelMove);
    register_callback(collector_context, c"GetMouseWheelMove".as_ptr(), Raylib_GetMouseWheelMove);
    register_callback(collector_context, c"get_mouse_wheel_move".as_ptr(), Raylib_GetMouseWheelMove);
    register_callback(collector_context, c"Raylib::GetMouseWheelMoveV".as_ptr(), Raylib_GetMouseWheelMoveV);
    register_callback(collector_context, c"Raylib::get_mouse_wheel_move_v".as_ptr(), Raylib_GetMouseWheelMoveV);
    register_callback(collector_context, c"raylib::GetMouseWheelMoveV".as_ptr(), Raylib_GetMouseWheelMoveV);
    register_callback(collector_context, c"raylib::get_mouse_wheel_move_v".as_ptr(), Raylib_GetMouseWheelMoveV);
    register_callback(collector_context, c"GetMouseWheelMoveV".as_ptr(), Raylib_GetMouseWheelMoveV);
    register_callback(collector_context, c"get_mouse_wheel_move_v".as_ptr(), Raylib_GetMouseWheelMoveV);
    register_callback(collector_context, c"Raylib::SetMouseCursor".as_ptr(), Raylib_SetMouseCursor);
    register_callback(collector_context, c"Raylib::set_mouse_cursor".as_ptr(), Raylib_SetMouseCursor);
    register_callback(collector_context, c"raylib::SetMouseCursor".as_ptr(), Raylib_SetMouseCursor);
    register_callback(collector_context, c"raylib::set_mouse_cursor".as_ptr(), Raylib_SetMouseCursor);
    register_callback(collector_context, c"SetMouseCursor".as_ptr(), Raylib_SetMouseCursor);
    register_callback(collector_context, c"set_mouse_cursor".as_ptr(), Raylib_SetMouseCursor);
    register_callback(collector_context, c"Raylib::GetTouchX".as_ptr(), Raylib_GetTouchX);
    register_callback(collector_context, c"Raylib::get_touch_x".as_ptr(), Raylib_GetTouchX);
    register_callback(collector_context, c"raylib::GetTouchX".as_ptr(), Raylib_GetTouchX);
    register_callback(collector_context, c"raylib::get_touch_x".as_ptr(), Raylib_GetTouchX);
    register_callback(collector_context, c"GetTouchX".as_ptr(), Raylib_GetTouchX);
    register_callback(collector_context, c"get_touch_x".as_ptr(), Raylib_GetTouchX);
    register_callback(collector_context, c"Raylib::GetTouchY".as_ptr(), Raylib_GetTouchY);
    register_callback(collector_context, c"Raylib::get_touch_y".as_ptr(), Raylib_GetTouchY);
    register_callback(collector_context, c"raylib::GetTouchY".as_ptr(), Raylib_GetTouchY);
    register_callback(collector_context, c"raylib::get_touch_y".as_ptr(), Raylib_GetTouchY);
    register_callback(collector_context, c"GetTouchY".as_ptr(), Raylib_GetTouchY);
    register_callback(collector_context, c"get_touch_y".as_ptr(), Raylib_GetTouchY);
    register_callback(collector_context, c"Raylib::GetTouchPosition".as_ptr(), Raylib_GetTouchPosition);
    register_callback(collector_context, c"Raylib::get_touch_position".as_ptr(), Raylib_GetTouchPosition);
    register_callback(collector_context, c"raylib::GetTouchPosition".as_ptr(), Raylib_GetTouchPosition);
    register_callback(collector_context, c"raylib::get_touch_position".as_ptr(), Raylib_GetTouchPosition);
    register_callback(collector_context, c"GetTouchPosition".as_ptr(), Raylib_GetTouchPosition);
    register_callback(collector_context, c"get_touch_position".as_ptr(), Raylib_GetTouchPosition);
    register_callback(collector_context, c"Raylib::GetTouchPointId".as_ptr(), Raylib_GetTouchPointId);
    register_callback(collector_context, c"Raylib::get_touch_point_id".as_ptr(), Raylib_GetTouchPointId);
    register_callback(collector_context, c"raylib::GetTouchPointId".as_ptr(), Raylib_GetTouchPointId);
    register_callback(collector_context, c"raylib::get_touch_point_id".as_ptr(), Raylib_GetTouchPointId);
    register_callback(collector_context, c"GetTouchPointId".as_ptr(), Raylib_GetTouchPointId);
    register_callback(collector_context, c"get_touch_point_id".as_ptr(), Raylib_GetTouchPointId);
    register_callback(collector_context, c"Raylib::GetTouchPointCount".as_ptr(), Raylib_GetTouchPointCount);
    register_callback(collector_context, c"Raylib::get_touch_point_count".as_ptr(), Raylib_GetTouchPointCount);
    register_callback(collector_context, c"raylib::GetTouchPointCount".as_ptr(), Raylib_GetTouchPointCount);
    register_callback(collector_context, c"raylib::get_touch_point_count".as_ptr(), Raylib_GetTouchPointCount);
    register_callback(collector_context, c"GetTouchPointCount".as_ptr(), Raylib_GetTouchPointCount);
    register_callback(collector_context, c"get_touch_point_count".as_ptr(), Raylib_GetTouchPointCount);
    register_callback(collector_context, c"Raylib::SetGesturesEnabled".as_ptr(), Raylib_SetGesturesEnabled);
    register_callback(collector_context, c"Raylib::set_gestures_enabled".as_ptr(), Raylib_SetGesturesEnabled);
    register_callback(collector_context, c"raylib::SetGesturesEnabled".as_ptr(), Raylib_SetGesturesEnabled);
    register_callback(collector_context, c"raylib::set_gestures_enabled".as_ptr(), Raylib_SetGesturesEnabled);
    register_callback(collector_context, c"SetGesturesEnabled".as_ptr(), Raylib_SetGesturesEnabled);
    register_callback(collector_context, c"set_gestures_enabled".as_ptr(), Raylib_SetGesturesEnabled);
    register_callback(collector_context, c"Raylib::IsGestureDetected".as_ptr(), Raylib_IsGestureDetected);
    register_callback(collector_context, c"Raylib::is_gesture_detected".as_ptr(), Raylib_IsGestureDetected);
    register_callback(collector_context, c"raylib::IsGestureDetected".as_ptr(), Raylib_IsGestureDetected);
    register_callback(collector_context, c"raylib::is_gesture_detected".as_ptr(), Raylib_IsGestureDetected);
    register_callback(collector_context, c"IsGestureDetected".as_ptr(), Raylib_IsGestureDetected);
    register_callback(collector_context, c"is_gesture_detected".as_ptr(), Raylib_IsGestureDetected);
    register_callback(collector_context, c"Raylib::GetGestureDetected".as_ptr(), Raylib_GetGestureDetected);
    register_callback(collector_context, c"Raylib::get_gesture_detected".as_ptr(), Raylib_GetGestureDetected);
    register_callback(collector_context, c"raylib::GetGestureDetected".as_ptr(), Raylib_GetGestureDetected);
    register_callback(collector_context, c"raylib::get_gesture_detected".as_ptr(), Raylib_GetGestureDetected);
    register_callback(collector_context, c"GetGestureDetected".as_ptr(), Raylib_GetGestureDetected);
    register_callback(collector_context, c"get_gesture_detected".as_ptr(), Raylib_GetGestureDetected);
    register_callback(collector_context, c"Raylib::GetGestureHoldDuration".as_ptr(), Raylib_GetGestureHoldDuration);
    register_callback(collector_context, c"Raylib::get_gesture_hold_duration".as_ptr(), Raylib_GetGestureHoldDuration);
    register_callback(collector_context, c"raylib::GetGestureHoldDuration".as_ptr(), Raylib_GetGestureHoldDuration);
    register_callback(collector_context, c"raylib::get_gesture_hold_duration".as_ptr(), Raylib_GetGestureHoldDuration);
    register_callback(collector_context, c"GetGestureHoldDuration".as_ptr(), Raylib_GetGestureHoldDuration);
    register_callback(collector_context, c"get_gesture_hold_duration".as_ptr(), Raylib_GetGestureHoldDuration);
    register_callback(collector_context, c"Raylib::GetGestureDragVector".as_ptr(), Raylib_GetGestureDragVector);
    register_callback(collector_context, c"Raylib::get_gesture_drag_vector".as_ptr(), Raylib_GetGestureDragVector);
    register_callback(collector_context, c"raylib::GetGestureDragVector".as_ptr(), Raylib_GetGestureDragVector);
    register_callback(collector_context, c"raylib::get_gesture_drag_vector".as_ptr(), Raylib_GetGestureDragVector);
    register_callback(collector_context, c"GetGestureDragVector".as_ptr(), Raylib_GetGestureDragVector);
    register_callback(collector_context, c"get_gesture_drag_vector".as_ptr(), Raylib_GetGestureDragVector);
    register_callback(collector_context, c"Raylib::GetGestureDragAngle".as_ptr(), Raylib_GetGestureDragAngle);
    register_callback(collector_context, c"Raylib::get_gesture_drag_angle".as_ptr(), Raylib_GetGestureDragAngle);
    register_callback(collector_context, c"raylib::GetGestureDragAngle".as_ptr(), Raylib_GetGestureDragAngle);
    register_callback(collector_context, c"raylib::get_gesture_drag_angle".as_ptr(), Raylib_GetGestureDragAngle);
    register_callback(collector_context, c"GetGestureDragAngle".as_ptr(), Raylib_GetGestureDragAngle);
    register_callback(collector_context, c"get_gesture_drag_angle".as_ptr(), Raylib_GetGestureDragAngle);
    register_callback(collector_context, c"Raylib::GetGesturePinchVector".as_ptr(), Raylib_GetGesturePinchVector);
    register_callback(collector_context, c"Raylib::get_gesture_pinch_vector".as_ptr(), Raylib_GetGesturePinchVector);
    register_callback(collector_context, c"raylib::GetGesturePinchVector".as_ptr(), Raylib_GetGesturePinchVector);
    register_callback(collector_context, c"raylib::get_gesture_pinch_vector".as_ptr(), Raylib_GetGesturePinchVector);
    register_callback(collector_context, c"GetGesturePinchVector".as_ptr(), Raylib_GetGesturePinchVector);
    register_callback(collector_context, c"get_gesture_pinch_vector".as_ptr(), Raylib_GetGesturePinchVector);
    register_callback(collector_context, c"Raylib::GetGesturePinchAngle".as_ptr(), Raylib_GetGesturePinchAngle);
    register_callback(collector_context, c"Raylib::get_gesture_pinch_angle".as_ptr(), Raylib_GetGesturePinchAngle);
    register_callback(collector_context, c"raylib::GetGesturePinchAngle".as_ptr(), Raylib_GetGesturePinchAngle);
    register_callback(collector_context, c"raylib::get_gesture_pinch_angle".as_ptr(), Raylib_GetGesturePinchAngle);
    register_callback(collector_context, c"GetGesturePinchAngle".as_ptr(), Raylib_GetGesturePinchAngle);
    register_callback(collector_context, c"get_gesture_pinch_angle".as_ptr(), Raylib_GetGesturePinchAngle);
    register_callback(collector_context, c"Raylib::UpdateCamera".as_ptr(), Raylib_UpdateCamera);
    register_callback(collector_context, c"Raylib::update_camera".as_ptr(), Raylib_UpdateCamera);
    register_callback(collector_context, c"raylib::UpdateCamera".as_ptr(), Raylib_UpdateCamera);
    register_callback(collector_context, c"raylib::update_camera".as_ptr(), Raylib_UpdateCamera);
    register_callback(collector_context, c"UpdateCamera".as_ptr(), Raylib_UpdateCamera);
    register_callback(collector_context, c"update_camera".as_ptr(), Raylib_UpdateCamera);
    register_callback(collector_context, c"Raylib::UpdateCameraPro".as_ptr(), Raylib_UpdateCameraPro);
    register_callback(collector_context, c"Raylib::update_camera_pro".as_ptr(), Raylib_UpdateCameraPro);
    register_callback(collector_context, c"raylib::UpdateCameraPro".as_ptr(), Raylib_UpdateCameraPro);
    register_callback(collector_context, c"raylib::update_camera_pro".as_ptr(), Raylib_UpdateCameraPro);
    register_callback(collector_context, c"UpdateCameraPro".as_ptr(), Raylib_UpdateCameraPro);
    register_callback(collector_context, c"update_camera_pro".as_ptr(), Raylib_UpdateCameraPro);
    register_callback(collector_context, c"Raylib::SetShapesTexture".as_ptr(), Raylib_SetShapesTexture);
    register_callback(collector_context, c"Raylib::set_shapes_texture".as_ptr(), Raylib_SetShapesTexture);
    register_callback(collector_context, c"raylib::SetShapesTexture".as_ptr(), Raylib_SetShapesTexture);
    register_callback(collector_context, c"raylib::set_shapes_texture".as_ptr(), Raylib_SetShapesTexture);
    register_callback(collector_context, c"SetShapesTexture".as_ptr(), Raylib_SetShapesTexture);
    register_callback(collector_context, c"set_shapes_texture".as_ptr(), Raylib_SetShapesTexture);
    register_callback(collector_context, c"Raylib::GetShapesTexture".as_ptr(), Raylib_GetShapesTexture);
    register_callback(collector_context, c"Raylib::get_shapes_texture".as_ptr(), Raylib_GetShapesTexture);
    register_callback(collector_context, c"raylib::GetShapesTexture".as_ptr(), Raylib_GetShapesTexture);
    register_callback(collector_context, c"raylib::get_shapes_texture".as_ptr(), Raylib_GetShapesTexture);
    register_callback(collector_context, c"GetShapesTexture".as_ptr(), Raylib_GetShapesTexture);
    register_callback(collector_context, c"get_shapes_texture".as_ptr(), Raylib_GetShapesTexture);
    register_callback(collector_context, c"Raylib::GetShapesTextureRectangle".as_ptr(), Raylib_GetShapesTextureRectangle);
    register_callback(collector_context, c"Raylib::get_shapes_texture_rectangle".as_ptr(), Raylib_GetShapesTextureRectangle);
    register_callback(collector_context, c"raylib::GetShapesTextureRectangle".as_ptr(), Raylib_GetShapesTextureRectangle);
    register_callback(collector_context, c"raylib::get_shapes_texture_rectangle".as_ptr(), Raylib_GetShapesTextureRectangle);
    register_callback(collector_context, c"GetShapesTextureRectangle".as_ptr(), Raylib_GetShapesTextureRectangle);
    register_callback(collector_context, c"get_shapes_texture_rectangle".as_ptr(), Raylib_GetShapesTextureRectangle);
    register_callback(collector_context, c"Raylib::DrawPixel".as_ptr(), Raylib_DrawPixel);
    register_callback(collector_context, c"Raylib::draw_pixel".as_ptr(), Raylib_DrawPixel);
    register_callback(collector_context, c"raylib::DrawPixel".as_ptr(), Raylib_DrawPixel);
    register_callback(collector_context, c"raylib::draw_pixel".as_ptr(), Raylib_DrawPixel);
    register_callback(collector_context, c"DrawPixel".as_ptr(), Raylib_DrawPixel);
    register_callback(collector_context, c"draw_pixel".as_ptr(), Raylib_DrawPixel);
    register_callback(collector_context, c"Raylib::DrawPixelV".as_ptr(), Raylib_DrawPixelV);
    register_callback(collector_context, c"Raylib::draw_pixel_v".as_ptr(), Raylib_DrawPixelV);
    register_callback(collector_context, c"raylib::DrawPixelV".as_ptr(), Raylib_DrawPixelV);
    register_callback(collector_context, c"raylib::draw_pixel_v".as_ptr(), Raylib_DrawPixelV);
    register_callback(collector_context, c"DrawPixelV".as_ptr(), Raylib_DrawPixelV);
    register_callback(collector_context, c"draw_pixel_v".as_ptr(), Raylib_DrawPixelV);
    register_callback(collector_context, c"Raylib::DrawLine".as_ptr(), Raylib_DrawLine);
    register_callback(collector_context, c"Raylib::draw_line".as_ptr(), Raylib_DrawLine);
    register_callback(collector_context, c"raylib::DrawLine".as_ptr(), Raylib_DrawLine);
    register_callback(collector_context, c"raylib::draw_line".as_ptr(), Raylib_DrawLine);
    register_callback(collector_context, c"DrawLine".as_ptr(), Raylib_DrawLine);
    register_callback(collector_context, c"draw_line".as_ptr(), Raylib_DrawLine);
    register_callback(collector_context, c"Raylib::DrawLineV".as_ptr(), Raylib_DrawLineV);
    register_callback(collector_context, c"Raylib::draw_line_v".as_ptr(), Raylib_DrawLineV);
    register_callback(collector_context, c"raylib::DrawLineV".as_ptr(), Raylib_DrawLineV);
    register_callback(collector_context, c"raylib::draw_line_v".as_ptr(), Raylib_DrawLineV);
    register_callback(collector_context, c"DrawLineV".as_ptr(), Raylib_DrawLineV);
    register_callback(collector_context, c"draw_line_v".as_ptr(), Raylib_DrawLineV);
    register_callback(collector_context, c"Raylib::DrawLineEx".as_ptr(), Raylib_DrawLineEx);
    register_callback(collector_context, c"Raylib::draw_line_ex".as_ptr(), Raylib_DrawLineEx);
    register_callback(collector_context, c"raylib::DrawLineEx".as_ptr(), Raylib_DrawLineEx);
    register_callback(collector_context, c"raylib::draw_line_ex".as_ptr(), Raylib_DrawLineEx);
    register_callback(collector_context, c"DrawLineEx".as_ptr(), Raylib_DrawLineEx);
    register_callback(collector_context, c"draw_line_ex".as_ptr(), Raylib_DrawLineEx);
    register_callback(collector_context, c"Raylib::DrawLineStrip".as_ptr(), Raylib_DrawLineStrip);
    register_callback(collector_context, c"Raylib::draw_line_strip".as_ptr(), Raylib_DrawLineStrip);
    register_callback(collector_context, c"raylib::DrawLineStrip".as_ptr(), Raylib_DrawLineStrip);
    register_callback(collector_context, c"raylib::draw_line_strip".as_ptr(), Raylib_DrawLineStrip);
    register_callback(collector_context, c"DrawLineStrip".as_ptr(), Raylib_DrawLineStrip);
    register_callback(collector_context, c"draw_line_strip".as_ptr(), Raylib_DrawLineStrip);
    register_callback(collector_context, c"Raylib::DrawLineBezier".as_ptr(), Raylib_DrawLineBezier);
    register_callback(collector_context, c"Raylib::draw_line_bezier".as_ptr(), Raylib_DrawLineBezier);
    register_callback(collector_context, c"raylib::DrawLineBezier".as_ptr(), Raylib_DrawLineBezier);
    register_callback(collector_context, c"raylib::draw_line_bezier".as_ptr(), Raylib_DrawLineBezier);
    register_callback(collector_context, c"DrawLineBezier".as_ptr(), Raylib_DrawLineBezier);
    register_callback(collector_context, c"draw_line_bezier".as_ptr(), Raylib_DrawLineBezier);
    register_callback(collector_context, c"Raylib::DrawLineDashed".as_ptr(), Raylib_DrawLineDashed);
    register_callback(collector_context, c"Raylib::draw_line_dashed".as_ptr(), Raylib_DrawLineDashed);
    register_callback(collector_context, c"raylib::DrawLineDashed".as_ptr(), Raylib_DrawLineDashed);
    register_callback(collector_context, c"raylib::draw_line_dashed".as_ptr(), Raylib_DrawLineDashed);
    register_callback(collector_context, c"DrawLineDashed".as_ptr(), Raylib_DrawLineDashed);
    register_callback(collector_context, c"draw_line_dashed".as_ptr(), Raylib_DrawLineDashed);
    register_callback(collector_context, c"Raylib::DrawTriangle".as_ptr(), Raylib_DrawTriangle);
    register_callback(collector_context, c"Raylib::draw_triangle".as_ptr(), Raylib_DrawTriangle);
    register_callback(collector_context, c"raylib::DrawTriangle".as_ptr(), Raylib_DrawTriangle);
    register_callback(collector_context, c"raylib::draw_triangle".as_ptr(), Raylib_DrawTriangle);
    register_callback(collector_context, c"DrawTriangle".as_ptr(), Raylib_DrawTriangle);
    register_callback(collector_context, c"draw_triangle".as_ptr(), Raylib_DrawTriangle);
    register_callback(collector_context, c"Raylib::DrawTriangleGradient".as_ptr(), Raylib_DrawTriangleGradient);
    register_callback(collector_context, c"Raylib::draw_triangle_gradient".as_ptr(), Raylib_DrawTriangleGradient);
    register_callback(collector_context, c"raylib::DrawTriangleGradient".as_ptr(), Raylib_DrawTriangleGradient);
    register_callback(collector_context, c"raylib::draw_triangle_gradient".as_ptr(), Raylib_DrawTriangleGradient);
    register_callback(collector_context, c"DrawTriangleGradient".as_ptr(), Raylib_DrawTriangleGradient);
    register_callback(collector_context, c"draw_triangle_gradient".as_ptr(), Raylib_DrawTriangleGradient);
    register_callback(collector_context, c"Raylib::DrawTriangleLines".as_ptr(), Raylib_DrawTriangleLines);
    register_callback(collector_context, c"Raylib::draw_triangle_lines".as_ptr(), Raylib_DrawTriangleLines);
    register_callback(collector_context, c"raylib::DrawTriangleLines".as_ptr(), Raylib_DrawTriangleLines);
    register_callback(collector_context, c"raylib::draw_triangle_lines".as_ptr(), Raylib_DrawTriangleLines);
    register_callback(collector_context, c"DrawTriangleLines".as_ptr(), Raylib_DrawTriangleLines);
    register_callback(collector_context, c"draw_triangle_lines".as_ptr(), Raylib_DrawTriangleLines);
    register_callback(collector_context, c"Raylib::DrawTriangleLinesEx".as_ptr(), Raylib_DrawTriangleLinesEx);
    register_callback(collector_context, c"Raylib::draw_triangle_lines_ex".as_ptr(), Raylib_DrawTriangleLinesEx);
    register_callback(collector_context, c"raylib::DrawTriangleLinesEx".as_ptr(), Raylib_DrawTriangleLinesEx);
    register_callback(collector_context, c"raylib::draw_triangle_lines_ex".as_ptr(), Raylib_DrawTriangleLinesEx);
    register_callback(collector_context, c"DrawTriangleLinesEx".as_ptr(), Raylib_DrawTriangleLinesEx);
    register_callback(collector_context, c"draw_triangle_lines_ex".as_ptr(), Raylib_DrawTriangleLinesEx);
    register_callback(collector_context, c"Raylib::DrawTriangleFan".as_ptr(), Raylib_DrawTriangleFan);
    register_callback(collector_context, c"Raylib::draw_triangle_fan".as_ptr(), Raylib_DrawTriangleFan);
    register_callback(collector_context, c"raylib::DrawTriangleFan".as_ptr(), Raylib_DrawTriangleFan);
    register_callback(collector_context, c"raylib::draw_triangle_fan".as_ptr(), Raylib_DrawTriangleFan);
    register_callback(collector_context, c"DrawTriangleFan".as_ptr(), Raylib_DrawTriangleFan);
    register_callback(collector_context, c"draw_triangle_fan".as_ptr(), Raylib_DrawTriangleFan);
    register_callback(collector_context, c"Raylib::DrawTriangleStrip".as_ptr(), Raylib_DrawTriangleStrip);
    register_callback(collector_context, c"Raylib::draw_triangle_strip".as_ptr(), Raylib_DrawTriangleStrip);
    register_callback(collector_context, c"raylib::DrawTriangleStrip".as_ptr(), Raylib_DrawTriangleStrip);
    register_callback(collector_context, c"raylib::draw_triangle_strip".as_ptr(), Raylib_DrawTriangleStrip);
    register_callback(collector_context, c"DrawTriangleStrip".as_ptr(), Raylib_DrawTriangleStrip);
    register_callback(collector_context, c"draw_triangle_strip".as_ptr(), Raylib_DrawTriangleStrip);
    register_callback(collector_context, c"Raylib::DrawRectangle".as_ptr(), Raylib_DrawRectangle);
    register_callback(collector_context, c"Raylib::draw_rectangle".as_ptr(), Raylib_DrawRectangle);
    register_callback(collector_context, c"raylib::DrawRectangle".as_ptr(), Raylib_DrawRectangle);
    register_callback(collector_context, c"raylib::draw_rectangle".as_ptr(), Raylib_DrawRectangle);
    register_callback(collector_context, c"DrawRectangle".as_ptr(), Raylib_DrawRectangle);
    register_callback(collector_context, c"draw_rectangle".as_ptr(), Raylib_DrawRectangle);
    register_callback(collector_context, c"Raylib::DrawRectangleV".as_ptr(), Raylib_DrawRectangleV);
    register_callback(collector_context, c"Raylib::draw_rectangle_v".as_ptr(), Raylib_DrawRectangleV);
    register_callback(collector_context, c"raylib::DrawRectangleV".as_ptr(), Raylib_DrawRectangleV);
    register_callback(collector_context, c"raylib::draw_rectangle_v".as_ptr(), Raylib_DrawRectangleV);
    register_callback(collector_context, c"DrawRectangleV".as_ptr(), Raylib_DrawRectangleV);
    register_callback(collector_context, c"draw_rectangle_v".as_ptr(), Raylib_DrawRectangleV);
    register_callback(collector_context, c"Raylib::DrawRectangleRec".as_ptr(), Raylib_DrawRectangleRec);
    register_callback(collector_context, c"Raylib::draw_rectangle_rec".as_ptr(), Raylib_DrawRectangleRec);
    register_callback(collector_context, c"raylib::DrawRectangleRec".as_ptr(), Raylib_DrawRectangleRec);
    register_callback(collector_context, c"raylib::draw_rectangle_rec".as_ptr(), Raylib_DrawRectangleRec);
    register_callback(collector_context, c"DrawRectangleRec".as_ptr(), Raylib_DrawRectangleRec);
    register_callback(collector_context, c"draw_rectangle_rec".as_ptr(), Raylib_DrawRectangleRec);
    register_callback(collector_context, c"Raylib::DrawRectanglePro".as_ptr(), Raylib_DrawRectanglePro);
    register_callback(collector_context, c"Raylib::draw_rectangle_pro".as_ptr(), Raylib_DrawRectanglePro);
    register_callback(collector_context, c"raylib::DrawRectanglePro".as_ptr(), Raylib_DrawRectanglePro);
    register_callback(collector_context, c"raylib::draw_rectangle_pro".as_ptr(), Raylib_DrawRectanglePro);
    register_callback(collector_context, c"DrawRectanglePro".as_ptr(), Raylib_DrawRectanglePro);
    register_callback(collector_context, c"draw_rectangle_pro".as_ptr(), Raylib_DrawRectanglePro);
    register_callback(collector_context, c"Raylib::DrawRectangleGradientV".as_ptr(), Raylib_DrawRectangleGradientV);
    register_callback(collector_context, c"Raylib::draw_rectangle_gradient_v".as_ptr(), Raylib_DrawRectangleGradientV);
    register_callback(collector_context, c"raylib::DrawRectangleGradientV".as_ptr(), Raylib_DrawRectangleGradientV);
    register_callback(collector_context, c"raylib::draw_rectangle_gradient_v".as_ptr(), Raylib_DrawRectangleGradientV);
    register_callback(collector_context, c"DrawRectangleGradientV".as_ptr(), Raylib_DrawRectangleGradientV);
    register_callback(collector_context, c"draw_rectangle_gradient_v".as_ptr(), Raylib_DrawRectangleGradientV);
    register_callback(collector_context, c"Raylib::DrawRectangleGradientH".as_ptr(), Raylib_DrawRectangleGradientH);
    register_callback(collector_context, c"Raylib::draw_rectangle_gradient_h".as_ptr(), Raylib_DrawRectangleGradientH);
    register_callback(collector_context, c"raylib::DrawRectangleGradientH".as_ptr(), Raylib_DrawRectangleGradientH);
    register_callback(collector_context, c"raylib::draw_rectangle_gradient_h".as_ptr(), Raylib_DrawRectangleGradientH);
    register_callback(collector_context, c"DrawRectangleGradientH".as_ptr(), Raylib_DrawRectangleGradientH);
    register_callback(collector_context, c"draw_rectangle_gradient_h".as_ptr(), Raylib_DrawRectangleGradientH);
    register_callback(collector_context, c"Raylib::DrawRectangleGradientEx".as_ptr(), Raylib_DrawRectangleGradientEx);
    register_callback(collector_context, c"Raylib::draw_rectangle_gradient_ex".as_ptr(), Raylib_DrawRectangleGradientEx);
    register_callback(collector_context, c"raylib::DrawRectangleGradientEx".as_ptr(), Raylib_DrawRectangleGradientEx);
    register_callback(collector_context, c"raylib::draw_rectangle_gradient_ex".as_ptr(), Raylib_DrawRectangleGradientEx);
    register_callback(collector_context, c"DrawRectangleGradientEx".as_ptr(), Raylib_DrawRectangleGradientEx);
    register_callback(collector_context, c"draw_rectangle_gradient_ex".as_ptr(), Raylib_DrawRectangleGradientEx);
    register_callback(collector_context, c"Raylib::DrawRectangleLines".as_ptr(), Raylib_DrawRectangleLines);
    register_callback(collector_context, c"Raylib::draw_rectangle_lines".as_ptr(), Raylib_DrawRectangleLines);
    register_callback(collector_context, c"raylib::DrawRectangleLines".as_ptr(), Raylib_DrawRectangleLines);
    register_callback(collector_context, c"raylib::draw_rectangle_lines".as_ptr(), Raylib_DrawRectangleLines);
    register_callback(collector_context, c"DrawRectangleLines".as_ptr(), Raylib_DrawRectangleLines);
    register_callback(collector_context, c"draw_rectangle_lines".as_ptr(), Raylib_DrawRectangleLines);
    register_callback(collector_context, c"Raylib::DrawRectangleLinesEx".as_ptr(), Raylib_DrawRectangleLinesEx);
    register_callback(collector_context, c"Raylib::draw_rectangle_lines_ex".as_ptr(), Raylib_DrawRectangleLinesEx);
    register_callback(collector_context, c"raylib::DrawRectangleLinesEx".as_ptr(), Raylib_DrawRectangleLinesEx);
    register_callback(collector_context, c"raylib::draw_rectangle_lines_ex".as_ptr(), Raylib_DrawRectangleLinesEx);
    register_callback(collector_context, c"DrawRectangleLinesEx".as_ptr(), Raylib_DrawRectangleLinesEx);
    register_callback(collector_context, c"draw_rectangle_lines_ex".as_ptr(), Raylib_DrawRectangleLinesEx);
    register_callback(collector_context, c"Raylib::DrawRectangleRounded".as_ptr(), Raylib_DrawRectangleRounded);
    register_callback(collector_context, c"Raylib::draw_rectangle_rounded".as_ptr(), Raylib_DrawRectangleRounded);
    register_callback(collector_context, c"raylib::DrawRectangleRounded".as_ptr(), Raylib_DrawRectangleRounded);
    register_callback(collector_context, c"raylib::draw_rectangle_rounded".as_ptr(), Raylib_DrawRectangleRounded);
    register_callback(collector_context, c"DrawRectangleRounded".as_ptr(), Raylib_DrawRectangleRounded);
    register_callback(collector_context, c"draw_rectangle_rounded".as_ptr(), Raylib_DrawRectangleRounded);
    register_callback(collector_context, c"Raylib::DrawRectangleRoundedLines".as_ptr(), Raylib_DrawRectangleRoundedLines);
    register_callback(collector_context, c"Raylib::draw_rectangle_rounded_lines".as_ptr(), Raylib_DrawRectangleRoundedLines);
    register_callback(collector_context, c"raylib::DrawRectangleRoundedLines".as_ptr(), Raylib_DrawRectangleRoundedLines);
    register_callback(collector_context, c"raylib::draw_rectangle_rounded_lines".as_ptr(), Raylib_DrawRectangleRoundedLines);
    register_callback(collector_context, c"DrawRectangleRoundedLines".as_ptr(), Raylib_DrawRectangleRoundedLines);
    register_callback(collector_context, c"draw_rectangle_rounded_lines".as_ptr(), Raylib_DrawRectangleRoundedLines);
    register_callback(collector_context, c"Raylib::DrawRectangleRoundedLinesEx".as_ptr(), Raylib_DrawRectangleRoundedLinesEx);
    register_callback(collector_context, c"Raylib::draw_rectangle_rounded_lines_ex".as_ptr(), Raylib_DrawRectangleRoundedLinesEx);
    register_callback(collector_context, c"raylib::DrawRectangleRoundedLinesEx".as_ptr(), Raylib_DrawRectangleRoundedLinesEx);
    register_callback(collector_context, c"raylib::draw_rectangle_rounded_lines_ex".as_ptr(), Raylib_DrawRectangleRoundedLinesEx);
    register_callback(collector_context, c"DrawRectangleRoundedLinesEx".as_ptr(), Raylib_DrawRectangleRoundedLinesEx);
    register_callback(collector_context, c"draw_rectangle_rounded_lines_ex".as_ptr(), Raylib_DrawRectangleRoundedLinesEx);
    register_callback(collector_context, c"Raylib::DrawPoly".as_ptr(), Raylib_DrawPoly);
    register_callback(collector_context, c"Raylib::draw_poly".as_ptr(), Raylib_DrawPoly);
    register_callback(collector_context, c"raylib::DrawPoly".as_ptr(), Raylib_DrawPoly);
    register_callback(collector_context, c"raylib::draw_poly".as_ptr(), Raylib_DrawPoly);
    register_callback(collector_context, c"DrawPoly".as_ptr(), Raylib_DrawPoly);
    register_callback(collector_context, c"draw_poly".as_ptr(), Raylib_DrawPoly);
    register_callback(collector_context, c"Raylib::DrawPolyLines".as_ptr(), Raylib_DrawPolyLines);
    register_callback(collector_context, c"Raylib::draw_poly_lines".as_ptr(), Raylib_DrawPolyLines);
    register_callback(collector_context, c"raylib::DrawPolyLines".as_ptr(), Raylib_DrawPolyLines);
    register_callback(collector_context, c"raylib::draw_poly_lines".as_ptr(), Raylib_DrawPolyLines);
    register_callback(collector_context, c"DrawPolyLines".as_ptr(), Raylib_DrawPolyLines);
    register_callback(collector_context, c"draw_poly_lines".as_ptr(), Raylib_DrawPolyLines);
    register_callback(collector_context, c"Raylib::DrawPolyLinesEx".as_ptr(), Raylib_DrawPolyLinesEx);
    register_callback(collector_context, c"Raylib::draw_poly_lines_ex".as_ptr(), Raylib_DrawPolyLinesEx);
    register_callback(collector_context, c"raylib::DrawPolyLinesEx".as_ptr(), Raylib_DrawPolyLinesEx);
    register_callback(collector_context, c"raylib::draw_poly_lines_ex".as_ptr(), Raylib_DrawPolyLinesEx);
    register_callback(collector_context, c"DrawPolyLinesEx".as_ptr(), Raylib_DrawPolyLinesEx);
    register_callback(collector_context, c"draw_poly_lines_ex".as_ptr(), Raylib_DrawPolyLinesEx);
    register_callback(collector_context, c"Raylib::DrawCircle".as_ptr(), Raylib_DrawCircle);
    register_callback(collector_context, c"Raylib::draw_circle".as_ptr(), Raylib_DrawCircle);
    register_callback(collector_context, c"raylib::DrawCircle".as_ptr(), Raylib_DrawCircle);
    register_callback(collector_context, c"raylib::draw_circle".as_ptr(), Raylib_DrawCircle);
    register_callback(collector_context, c"DrawCircle".as_ptr(), Raylib_DrawCircle);
    register_callback(collector_context, c"draw_circle".as_ptr(), Raylib_DrawCircle);
    register_callback(collector_context, c"Raylib::DrawCircleV".as_ptr(), Raylib_DrawCircleV);
    register_callback(collector_context, c"Raylib::draw_circle_v".as_ptr(), Raylib_DrawCircleV);
    register_callback(collector_context, c"raylib::DrawCircleV".as_ptr(), Raylib_DrawCircleV);
    register_callback(collector_context, c"raylib::draw_circle_v".as_ptr(), Raylib_DrawCircleV);
    register_callback(collector_context, c"DrawCircleV".as_ptr(), Raylib_DrawCircleV);
    register_callback(collector_context, c"draw_circle_v".as_ptr(), Raylib_DrawCircleV);
    register_callback(collector_context, c"Raylib::DrawCircleGradient".as_ptr(), Raylib_DrawCircleGradient);
    register_callback(collector_context, c"Raylib::draw_circle_gradient".as_ptr(), Raylib_DrawCircleGradient);
    register_callback(collector_context, c"raylib::DrawCircleGradient".as_ptr(), Raylib_DrawCircleGradient);
    register_callback(collector_context, c"raylib::draw_circle_gradient".as_ptr(), Raylib_DrawCircleGradient);
    register_callback(collector_context, c"DrawCircleGradient".as_ptr(), Raylib_DrawCircleGradient);
    register_callback(collector_context, c"draw_circle_gradient".as_ptr(), Raylib_DrawCircleGradient);
    register_callback(collector_context, c"Raylib::DrawCircleSector".as_ptr(), Raylib_DrawCircleSector);
    register_callback(collector_context, c"Raylib::draw_circle_sector".as_ptr(), Raylib_DrawCircleSector);
    register_callback(collector_context, c"raylib::DrawCircleSector".as_ptr(), Raylib_DrawCircleSector);
    register_callback(collector_context, c"raylib::draw_circle_sector".as_ptr(), Raylib_DrawCircleSector);
    register_callback(collector_context, c"DrawCircleSector".as_ptr(), Raylib_DrawCircleSector);
    register_callback(collector_context, c"draw_circle_sector".as_ptr(), Raylib_DrawCircleSector);
    register_callback(collector_context, c"Raylib::DrawCircleSectorLines".as_ptr(), Raylib_DrawCircleSectorLines);
    register_callback(collector_context, c"Raylib::draw_circle_sector_lines".as_ptr(), Raylib_DrawCircleSectorLines);
    register_callback(collector_context, c"raylib::DrawCircleSectorLines".as_ptr(), Raylib_DrawCircleSectorLines);
    register_callback(collector_context, c"raylib::draw_circle_sector_lines".as_ptr(), Raylib_DrawCircleSectorLines);
    register_callback(collector_context, c"DrawCircleSectorLines".as_ptr(), Raylib_DrawCircleSectorLines);
    register_callback(collector_context, c"draw_circle_sector_lines".as_ptr(), Raylib_DrawCircleSectorLines);
    register_callback(collector_context, c"Raylib::DrawCircleSectorLinesEx".as_ptr(), Raylib_DrawCircleSectorLinesEx);
    register_callback(collector_context, c"Raylib::draw_circle_sector_lines_ex".as_ptr(), Raylib_DrawCircleSectorLinesEx);
    register_callback(collector_context, c"raylib::DrawCircleSectorLinesEx".as_ptr(), Raylib_DrawCircleSectorLinesEx);
    register_callback(collector_context, c"raylib::draw_circle_sector_lines_ex".as_ptr(), Raylib_DrawCircleSectorLinesEx);
    register_callback(collector_context, c"DrawCircleSectorLinesEx".as_ptr(), Raylib_DrawCircleSectorLinesEx);
    register_callback(collector_context, c"draw_circle_sector_lines_ex".as_ptr(), Raylib_DrawCircleSectorLinesEx);
    register_callback(collector_context, c"Raylib::DrawCircleLines".as_ptr(), Raylib_DrawCircleLines);
    register_callback(collector_context, c"Raylib::draw_circle_lines".as_ptr(), Raylib_DrawCircleLines);
    register_callback(collector_context, c"raylib::DrawCircleLines".as_ptr(), Raylib_DrawCircleLines);
    register_callback(collector_context, c"raylib::draw_circle_lines".as_ptr(), Raylib_DrawCircleLines);
    register_callback(collector_context, c"DrawCircleLines".as_ptr(), Raylib_DrawCircleLines);
    register_callback(collector_context, c"draw_circle_lines".as_ptr(), Raylib_DrawCircleLines);
    register_callback(collector_context, c"Raylib::DrawCircleLinesV".as_ptr(), Raylib_DrawCircleLinesV);
    register_callback(collector_context, c"Raylib::draw_circle_lines_v".as_ptr(), Raylib_DrawCircleLinesV);
    register_callback(collector_context, c"raylib::DrawCircleLinesV".as_ptr(), Raylib_DrawCircleLinesV);
    register_callback(collector_context, c"raylib::draw_circle_lines_v".as_ptr(), Raylib_DrawCircleLinesV);
    register_callback(collector_context, c"DrawCircleLinesV".as_ptr(), Raylib_DrawCircleLinesV);
    register_callback(collector_context, c"draw_circle_lines_v".as_ptr(), Raylib_DrawCircleLinesV);
    register_callback(collector_context, c"Raylib::DrawCircleLinesEx".as_ptr(), Raylib_DrawCircleLinesEx);
    register_callback(collector_context, c"Raylib::draw_circle_lines_ex".as_ptr(), Raylib_DrawCircleLinesEx);
    register_callback(collector_context, c"raylib::DrawCircleLinesEx".as_ptr(), Raylib_DrawCircleLinesEx);
    register_callback(collector_context, c"raylib::draw_circle_lines_ex".as_ptr(), Raylib_DrawCircleLinesEx);
    register_callback(collector_context, c"DrawCircleLinesEx".as_ptr(), Raylib_DrawCircleLinesEx);
    register_callback(collector_context, c"draw_circle_lines_ex".as_ptr(), Raylib_DrawCircleLinesEx);
    register_callback(collector_context, c"Raylib::DrawEllipse".as_ptr(), Raylib_DrawEllipse);
    register_callback(collector_context, c"Raylib::draw_ellipse".as_ptr(), Raylib_DrawEllipse);
    register_callback(collector_context, c"raylib::DrawEllipse".as_ptr(), Raylib_DrawEllipse);
    register_callback(collector_context, c"raylib::draw_ellipse".as_ptr(), Raylib_DrawEllipse);
    register_callback(collector_context, c"DrawEllipse".as_ptr(), Raylib_DrawEllipse);
    register_callback(collector_context, c"draw_ellipse".as_ptr(), Raylib_DrawEllipse);
    register_callback(collector_context, c"Raylib::DrawEllipseV".as_ptr(), Raylib_DrawEllipseV);
    register_callback(collector_context, c"Raylib::draw_ellipse_v".as_ptr(), Raylib_DrawEllipseV);
    register_callback(collector_context, c"raylib::DrawEllipseV".as_ptr(), Raylib_DrawEllipseV);
    register_callback(collector_context, c"raylib::draw_ellipse_v".as_ptr(), Raylib_DrawEllipseV);
    register_callback(collector_context, c"DrawEllipseV".as_ptr(), Raylib_DrawEllipseV);
    register_callback(collector_context, c"draw_ellipse_v".as_ptr(), Raylib_DrawEllipseV);
    register_callback(collector_context, c"Raylib::DrawEllipseLines".as_ptr(), Raylib_DrawEllipseLines);
    register_callback(collector_context, c"Raylib::draw_ellipse_lines".as_ptr(), Raylib_DrawEllipseLines);
    register_callback(collector_context, c"raylib::DrawEllipseLines".as_ptr(), Raylib_DrawEllipseLines);
    register_callback(collector_context, c"raylib::draw_ellipse_lines".as_ptr(), Raylib_DrawEllipseLines);
    register_callback(collector_context, c"DrawEllipseLines".as_ptr(), Raylib_DrawEllipseLines);
    register_callback(collector_context, c"draw_ellipse_lines".as_ptr(), Raylib_DrawEllipseLines);
    register_callback(collector_context, c"Raylib::DrawEllipseLinesV".as_ptr(), Raylib_DrawEllipseLinesV);
    register_callback(collector_context, c"Raylib::draw_ellipse_lines_v".as_ptr(), Raylib_DrawEllipseLinesV);
    register_callback(collector_context, c"raylib::DrawEllipseLinesV".as_ptr(), Raylib_DrawEllipseLinesV);
    register_callback(collector_context, c"raylib::draw_ellipse_lines_v".as_ptr(), Raylib_DrawEllipseLinesV);
    register_callback(collector_context, c"DrawEllipseLinesV".as_ptr(), Raylib_DrawEllipseLinesV);
    register_callback(collector_context, c"draw_ellipse_lines_v".as_ptr(), Raylib_DrawEllipseLinesV);
    register_callback(collector_context, c"Raylib::DrawEllipseLinesEx".as_ptr(), Raylib_DrawEllipseLinesEx);
    register_callback(collector_context, c"Raylib::draw_ellipse_lines_ex".as_ptr(), Raylib_DrawEllipseLinesEx);
    register_callback(collector_context, c"raylib::DrawEllipseLinesEx".as_ptr(), Raylib_DrawEllipseLinesEx);
    register_callback(collector_context, c"raylib::draw_ellipse_lines_ex".as_ptr(), Raylib_DrawEllipseLinesEx);
    register_callback(collector_context, c"DrawEllipseLinesEx".as_ptr(), Raylib_DrawEllipseLinesEx);
    register_callback(collector_context, c"draw_ellipse_lines_ex".as_ptr(), Raylib_DrawEllipseLinesEx);
    register_callback(collector_context, c"Raylib::DrawRing".as_ptr(), Raylib_DrawRing);
    register_callback(collector_context, c"Raylib::draw_ring".as_ptr(), Raylib_DrawRing);
    register_callback(collector_context, c"raylib::DrawRing".as_ptr(), Raylib_DrawRing);
    register_callback(collector_context, c"raylib::draw_ring".as_ptr(), Raylib_DrawRing);
    register_callback(collector_context, c"DrawRing".as_ptr(), Raylib_DrawRing);
    register_callback(collector_context, c"draw_ring".as_ptr(), Raylib_DrawRing);
    register_callback(collector_context, c"Raylib::DrawRingLines".as_ptr(), Raylib_DrawRingLines);
    register_callback(collector_context, c"Raylib::draw_ring_lines".as_ptr(), Raylib_DrawRingLines);
    register_callback(collector_context, c"raylib::DrawRingLines".as_ptr(), Raylib_DrawRingLines);
    register_callback(collector_context, c"raylib::draw_ring_lines".as_ptr(), Raylib_DrawRingLines);
    register_callback(collector_context, c"DrawRingLines".as_ptr(), Raylib_DrawRingLines);
    register_callback(collector_context, c"draw_ring_lines".as_ptr(), Raylib_DrawRingLines);
    register_callback(collector_context, c"Raylib::DrawRingLinesEx".as_ptr(), Raylib_DrawRingLinesEx);
    register_callback(collector_context, c"Raylib::draw_ring_lines_ex".as_ptr(), Raylib_DrawRingLinesEx);
    register_callback(collector_context, c"raylib::DrawRingLinesEx".as_ptr(), Raylib_DrawRingLinesEx);
    register_callback(collector_context, c"raylib::draw_ring_lines_ex".as_ptr(), Raylib_DrawRingLinesEx);
    register_callback(collector_context, c"DrawRingLinesEx".as_ptr(), Raylib_DrawRingLinesEx);
    register_callback(collector_context, c"draw_ring_lines_ex".as_ptr(), Raylib_DrawRingLinesEx);
    register_callback(collector_context, c"Raylib::DrawSplineLinear".as_ptr(), Raylib_DrawSplineLinear);
    register_callback(collector_context, c"Raylib::draw_spline_linear".as_ptr(), Raylib_DrawSplineLinear);
    register_callback(collector_context, c"raylib::DrawSplineLinear".as_ptr(), Raylib_DrawSplineLinear);
    register_callback(collector_context, c"raylib::draw_spline_linear".as_ptr(), Raylib_DrawSplineLinear);
    register_callback(collector_context, c"DrawSplineLinear".as_ptr(), Raylib_DrawSplineLinear);
    register_callback(collector_context, c"draw_spline_linear".as_ptr(), Raylib_DrawSplineLinear);
    register_callback(collector_context, c"Raylib::DrawSplineBasis".as_ptr(), Raylib_DrawSplineBasis);
    register_callback(collector_context, c"Raylib::draw_spline_basis".as_ptr(), Raylib_DrawSplineBasis);
    register_callback(collector_context, c"raylib::DrawSplineBasis".as_ptr(), Raylib_DrawSplineBasis);
    register_callback(collector_context, c"raylib::draw_spline_basis".as_ptr(), Raylib_DrawSplineBasis);
    register_callback(collector_context, c"DrawSplineBasis".as_ptr(), Raylib_DrawSplineBasis);
    register_callback(collector_context, c"draw_spline_basis".as_ptr(), Raylib_DrawSplineBasis);
    register_callback(collector_context, c"Raylib::DrawSplineCatmullRom".as_ptr(), Raylib_DrawSplineCatmullRom);
    register_callback(collector_context, c"Raylib::draw_spline_catmull_rom".as_ptr(), Raylib_DrawSplineCatmullRom);
    register_callback(collector_context, c"raylib::DrawSplineCatmullRom".as_ptr(), Raylib_DrawSplineCatmullRom);
    register_callback(collector_context, c"raylib::draw_spline_catmull_rom".as_ptr(), Raylib_DrawSplineCatmullRom);
    register_callback(collector_context, c"DrawSplineCatmullRom".as_ptr(), Raylib_DrawSplineCatmullRom);
    register_callback(collector_context, c"draw_spline_catmull_rom".as_ptr(), Raylib_DrawSplineCatmullRom);
    register_callback(collector_context, c"Raylib::DrawSplineBezierQuadratic".as_ptr(), Raylib_DrawSplineBezierQuadratic);
    register_callback(collector_context, c"Raylib::draw_spline_bezier_quadratic".as_ptr(), Raylib_DrawSplineBezierQuadratic);
    register_callback(collector_context, c"raylib::DrawSplineBezierQuadratic".as_ptr(), Raylib_DrawSplineBezierQuadratic);
    register_callback(collector_context, c"raylib::draw_spline_bezier_quadratic".as_ptr(), Raylib_DrawSplineBezierQuadratic);
    register_callback(collector_context, c"DrawSplineBezierQuadratic".as_ptr(), Raylib_DrawSplineBezierQuadratic);
    register_callback(collector_context, c"draw_spline_bezier_quadratic".as_ptr(), Raylib_DrawSplineBezierQuadratic);
    register_callback(collector_context, c"Raylib::DrawSplineBezierCubic".as_ptr(), Raylib_DrawSplineBezierCubic);
    register_callback(collector_context, c"Raylib::draw_spline_bezier_cubic".as_ptr(), Raylib_DrawSplineBezierCubic);
    register_callback(collector_context, c"raylib::DrawSplineBezierCubic".as_ptr(), Raylib_DrawSplineBezierCubic);
    register_callback(collector_context, c"raylib::draw_spline_bezier_cubic".as_ptr(), Raylib_DrawSplineBezierCubic);
    register_callback(collector_context, c"DrawSplineBezierCubic".as_ptr(), Raylib_DrawSplineBezierCubic);
    register_callback(collector_context, c"draw_spline_bezier_cubic".as_ptr(), Raylib_DrawSplineBezierCubic);
    register_callback(collector_context, c"Raylib::DrawSplineSegmentLinear".as_ptr(), Raylib_DrawSplineSegmentLinear);
    register_callback(collector_context, c"Raylib::draw_spline_segment_linear".as_ptr(), Raylib_DrawSplineSegmentLinear);
    register_callback(collector_context, c"raylib::DrawSplineSegmentLinear".as_ptr(), Raylib_DrawSplineSegmentLinear);
    register_callback(collector_context, c"raylib::draw_spline_segment_linear".as_ptr(), Raylib_DrawSplineSegmentLinear);
    register_callback(collector_context, c"DrawSplineSegmentLinear".as_ptr(), Raylib_DrawSplineSegmentLinear);
    register_callback(collector_context, c"draw_spline_segment_linear".as_ptr(), Raylib_DrawSplineSegmentLinear);
    register_callback(collector_context, c"Raylib::DrawSplineSegmentBasis".as_ptr(), Raylib_DrawSplineSegmentBasis);
    register_callback(collector_context, c"Raylib::draw_spline_segment_basis".as_ptr(), Raylib_DrawSplineSegmentBasis);
    register_callback(collector_context, c"raylib::DrawSplineSegmentBasis".as_ptr(), Raylib_DrawSplineSegmentBasis);
    register_callback(collector_context, c"raylib::draw_spline_segment_basis".as_ptr(), Raylib_DrawSplineSegmentBasis);
    register_callback(collector_context, c"DrawSplineSegmentBasis".as_ptr(), Raylib_DrawSplineSegmentBasis);
    register_callback(collector_context, c"draw_spline_segment_basis".as_ptr(), Raylib_DrawSplineSegmentBasis);
    register_callback(collector_context, c"Raylib::DrawSplineSegmentCatmullRom".as_ptr(), Raylib_DrawSplineSegmentCatmullRom);
    register_callback(collector_context, c"Raylib::draw_spline_segment_catmull_rom".as_ptr(), Raylib_DrawSplineSegmentCatmullRom);
    register_callback(collector_context, c"raylib::DrawSplineSegmentCatmullRom".as_ptr(), Raylib_DrawSplineSegmentCatmullRom);
    register_callback(collector_context, c"raylib::draw_spline_segment_catmull_rom".as_ptr(), Raylib_DrawSplineSegmentCatmullRom);
    register_callback(collector_context, c"DrawSplineSegmentCatmullRom".as_ptr(), Raylib_DrawSplineSegmentCatmullRom);
    register_callback(collector_context, c"draw_spline_segment_catmull_rom".as_ptr(), Raylib_DrawSplineSegmentCatmullRom);
    register_callback(collector_context, c"Raylib::DrawSplineSegmentBezierQuadratic".as_ptr(), Raylib_DrawSplineSegmentBezierQuadratic);
    register_callback(collector_context, c"Raylib::draw_spline_segment_bezier_quadratic".as_ptr(), Raylib_DrawSplineSegmentBezierQuadratic);
    register_callback(collector_context, c"raylib::DrawSplineSegmentBezierQuadratic".as_ptr(), Raylib_DrawSplineSegmentBezierQuadratic);
    register_callback(collector_context, c"raylib::draw_spline_segment_bezier_quadratic".as_ptr(), Raylib_DrawSplineSegmentBezierQuadratic);
    register_callback(collector_context, c"DrawSplineSegmentBezierQuadratic".as_ptr(), Raylib_DrawSplineSegmentBezierQuadratic);
    register_callback(collector_context, c"draw_spline_segment_bezier_quadratic".as_ptr(), Raylib_DrawSplineSegmentBezierQuadratic);
    register_callback(collector_context, c"Raylib::DrawSplineSegmentBezierCubic".as_ptr(), Raylib_DrawSplineSegmentBezierCubic);
    register_callback(collector_context, c"Raylib::draw_spline_segment_bezier_cubic".as_ptr(), Raylib_DrawSplineSegmentBezierCubic);
    register_callback(collector_context, c"raylib::DrawSplineSegmentBezierCubic".as_ptr(), Raylib_DrawSplineSegmentBezierCubic);
    register_callback(collector_context, c"raylib::draw_spline_segment_bezier_cubic".as_ptr(), Raylib_DrawSplineSegmentBezierCubic);
    register_callback(collector_context, c"DrawSplineSegmentBezierCubic".as_ptr(), Raylib_DrawSplineSegmentBezierCubic);
    register_callback(collector_context, c"draw_spline_segment_bezier_cubic".as_ptr(), Raylib_DrawSplineSegmentBezierCubic);
    register_callback(collector_context, c"Raylib::GetSplinePointLinear".as_ptr(), Raylib_GetSplinePointLinear);
    register_callback(collector_context, c"Raylib::get_spline_point_linear".as_ptr(), Raylib_GetSplinePointLinear);
    register_callback(collector_context, c"raylib::GetSplinePointLinear".as_ptr(), Raylib_GetSplinePointLinear);
    register_callback(collector_context, c"raylib::get_spline_point_linear".as_ptr(), Raylib_GetSplinePointLinear);
    register_callback(collector_context, c"GetSplinePointLinear".as_ptr(), Raylib_GetSplinePointLinear);
    register_callback(collector_context, c"get_spline_point_linear".as_ptr(), Raylib_GetSplinePointLinear);
    register_callback(collector_context, c"Raylib::GetSplinePointBasis".as_ptr(), Raylib_GetSplinePointBasis);
    register_callback(collector_context, c"Raylib::get_spline_point_basis".as_ptr(), Raylib_GetSplinePointBasis);
    register_callback(collector_context, c"raylib::GetSplinePointBasis".as_ptr(), Raylib_GetSplinePointBasis);
    register_callback(collector_context, c"raylib::get_spline_point_basis".as_ptr(), Raylib_GetSplinePointBasis);
    register_callback(collector_context, c"GetSplinePointBasis".as_ptr(), Raylib_GetSplinePointBasis);
    register_callback(collector_context, c"get_spline_point_basis".as_ptr(), Raylib_GetSplinePointBasis);
    register_callback(collector_context, c"Raylib::GetSplinePointCatmullRom".as_ptr(), Raylib_GetSplinePointCatmullRom);
    register_callback(collector_context, c"Raylib::get_spline_point_catmull_rom".as_ptr(), Raylib_GetSplinePointCatmullRom);
    register_callback(collector_context, c"raylib::GetSplinePointCatmullRom".as_ptr(), Raylib_GetSplinePointCatmullRom);
    register_callback(collector_context, c"raylib::get_spline_point_catmull_rom".as_ptr(), Raylib_GetSplinePointCatmullRom);
    register_callback(collector_context, c"GetSplinePointCatmullRom".as_ptr(), Raylib_GetSplinePointCatmullRom);
    register_callback(collector_context, c"get_spline_point_catmull_rom".as_ptr(), Raylib_GetSplinePointCatmullRom);
    register_callback(collector_context, c"Raylib::GetSplinePointBezierQuadratic".as_ptr(), Raylib_GetSplinePointBezierQuadratic);
    register_callback(collector_context, c"Raylib::get_spline_point_bezier_quadratic".as_ptr(), Raylib_GetSplinePointBezierQuadratic);
    register_callback(collector_context, c"raylib::GetSplinePointBezierQuadratic".as_ptr(), Raylib_GetSplinePointBezierQuadratic);
    register_callback(collector_context, c"raylib::get_spline_point_bezier_quadratic".as_ptr(), Raylib_GetSplinePointBezierQuadratic);
    register_callback(collector_context, c"GetSplinePointBezierQuadratic".as_ptr(), Raylib_GetSplinePointBezierQuadratic);
    register_callback(collector_context, c"get_spline_point_bezier_quadratic".as_ptr(), Raylib_GetSplinePointBezierQuadratic);
    register_callback(collector_context, c"Raylib::GetSplinePointBezierCubic".as_ptr(), Raylib_GetSplinePointBezierCubic);
    register_callback(collector_context, c"Raylib::get_spline_point_bezier_cubic".as_ptr(), Raylib_GetSplinePointBezierCubic);
    register_callback(collector_context, c"raylib::GetSplinePointBezierCubic".as_ptr(), Raylib_GetSplinePointBezierCubic);
    register_callback(collector_context, c"raylib::get_spline_point_bezier_cubic".as_ptr(), Raylib_GetSplinePointBezierCubic);
    register_callback(collector_context, c"GetSplinePointBezierCubic".as_ptr(), Raylib_GetSplinePointBezierCubic);
    register_callback(collector_context, c"get_spline_point_bezier_cubic".as_ptr(), Raylib_GetSplinePointBezierCubic);
    register_callback(collector_context, c"Raylib::CheckCollisionRecs".as_ptr(), Raylib_CheckCollisionRecs);
    register_callback(collector_context, c"Raylib::check_collision_recs".as_ptr(), Raylib_CheckCollisionRecs);
    register_callback(collector_context, c"raylib::CheckCollisionRecs".as_ptr(), Raylib_CheckCollisionRecs);
    register_callback(collector_context, c"raylib::check_collision_recs".as_ptr(), Raylib_CheckCollisionRecs);
    register_callback(collector_context, c"CheckCollisionRecs".as_ptr(), Raylib_CheckCollisionRecs);
    register_callback(collector_context, c"check_collision_recs".as_ptr(), Raylib_CheckCollisionRecs);
    register_callback(collector_context, c"Raylib::CheckCollisionCircles".as_ptr(), Raylib_CheckCollisionCircles);
    register_callback(collector_context, c"Raylib::check_collision_circles".as_ptr(), Raylib_CheckCollisionCircles);
    register_callback(collector_context, c"raylib::CheckCollisionCircles".as_ptr(), Raylib_CheckCollisionCircles);
    register_callback(collector_context, c"raylib::check_collision_circles".as_ptr(), Raylib_CheckCollisionCircles);
    register_callback(collector_context, c"CheckCollisionCircles".as_ptr(), Raylib_CheckCollisionCircles);
    register_callback(collector_context, c"check_collision_circles".as_ptr(), Raylib_CheckCollisionCircles);
    register_callback(collector_context, c"Raylib::CheckCollisionCircleRec".as_ptr(), Raylib_CheckCollisionCircleRec);
    register_callback(collector_context, c"Raylib::check_collision_circle_rec".as_ptr(), Raylib_CheckCollisionCircleRec);
    register_callback(collector_context, c"raylib::CheckCollisionCircleRec".as_ptr(), Raylib_CheckCollisionCircleRec);
    register_callback(collector_context, c"raylib::check_collision_circle_rec".as_ptr(), Raylib_CheckCollisionCircleRec);
    register_callback(collector_context, c"CheckCollisionCircleRec".as_ptr(), Raylib_CheckCollisionCircleRec);
    register_callback(collector_context, c"check_collision_circle_rec".as_ptr(), Raylib_CheckCollisionCircleRec);
    register_callback(collector_context, c"Raylib::CheckCollisionCircleLine".as_ptr(), Raylib_CheckCollisionCircleLine);
    register_callback(collector_context, c"Raylib::check_collision_circle_line".as_ptr(), Raylib_CheckCollisionCircleLine);
    register_callback(collector_context, c"raylib::CheckCollisionCircleLine".as_ptr(), Raylib_CheckCollisionCircleLine);
    register_callback(collector_context, c"raylib::check_collision_circle_line".as_ptr(), Raylib_CheckCollisionCircleLine);
    register_callback(collector_context, c"CheckCollisionCircleLine".as_ptr(), Raylib_CheckCollisionCircleLine);
    register_callback(collector_context, c"check_collision_circle_line".as_ptr(), Raylib_CheckCollisionCircleLine);
    register_callback(collector_context, c"Raylib::CheckCollisionPointRec".as_ptr(), Raylib_CheckCollisionPointRec);
    register_callback(collector_context, c"Raylib::check_collision_point_rec".as_ptr(), Raylib_CheckCollisionPointRec);
    register_callback(collector_context, c"raylib::CheckCollisionPointRec".as_ptr(), Raylib_CheckCollisionPointRec);
    register_callback(collector_context, c"raylib::check_collision_point_rec".as_ptr(), Raylib_CheckCollisionPointRec);
    register_callback(collector_context, c"CheckCollisionPointRec".as_ptr(), Raylib_CheckCollisionPointRec);
    register_callback(collector_context, c"check_collision_point_rec".as_ptr(), Raylib_CheckCollisionPointRec);
    register_callback(collector_context, c"Raylib::CheckCollisionPointCircle".as_ptr(), Raylib_CheckCollisionPointCircle);
    register_callback(collector_context, c"Raylib::check_collision_point_circle".as_ptr(), Raylib_CheckCollisionPointCircle);
    register_callback(collector_context, c"raylib::CheckCollisionPointCircle".as_ptr(), Raylib_CheckCollisionPointCircle);
    register_callback(collector_context, c"raylib::check_collision_point_circle".as_ptr(), Raylib_CheckCollisionPointCircle);
    register_callback(collector_context, c"CheckCollisionPointCircle".as_ptr(), Raylib_CheckCollisionPointCircle);
    register_callback(collector_context, c"check_collision_point_circle".as_ptr(), Raylib_CheckCollisionPointCircle);
    register_callback(collector_context, c"Raylib::CheckCollisionPointTriangle".as_ptr(), Raylib_CheckCollisionPointTriangle);
    register_callback(collector_context, c"Raylib::check_collision_point_triangle".as_ptr(), Raylib_CheckCollisionPointTriangle);
    register_callback(collector_context, c"raylib::CheckCollisionPointTriangle".as_ptr(), Raylib_CheckCollisionPointTriangle);
    register_callback(collector_context, c"raylib::check_collision_point_triangle".as_ptr(), Raylib_CheckCollisionPointTriangle);
    register_callback(collector_context, c"CheckCollisionPointTriangle".as_ptr(), Raylib_CheckCollisionPointTriangle);
    register_callback(collector_context, c"check_collision_point_triangle".as_ptr(), Raylib_CheckCollisionPointTriangle);
    register_callback(collector_context, c"Raylib::CheckCollisionPointLine".as_ptr(), Raylib_CheckCollisionPointLine);
    register_callback(collector_context, c"Raylib::check_collision_point_line".as_ptr(), Raylib_CheckCollisionPointLine);
    register_callback(collector_context, c"raylib::CheckCollisionPointLine".as_ptr(), Raylib_CheckCollisionPointLine);
    register_callback(collector_context, c"raylib::check_collision_point_line".as_ptr(), Raylib_CheckCollisionPointLine);
    register_callback(collector_context, c"CheckCollisionPointLine".as_ptr(), Raylib_CheckCollisionPointLine);
    register_callback(collector_context, c"check_collision_point_line".as_ptr(), Raylib_CheckCollisionPointLine);
    register_callback(collector_context, c"Raylib::CheckCollisionPointPoly".as_ptr(), Raylib_CheckCollisionPointPoly);
    register_callback(collector_context, c"Raylib::check_collision_point_poly".as_ptr(), Raylib_CheckCollisionPointPoly);
    register_callback(collector_context, c"raylib::CheckCollisionPointPoly".as_ptr(), Raylib_CheckCollisionPointPoly);
    register_callback(collector_context, c"raylib::check_collision_point_poly".as_ptr(), Raylib_CheckCollisionPointPoly);
    register_callback(collector_context, c"CheckCollisionPointPoly".as_ptr(), Raylib_CheckCollisionPointPoly);
    register_callback(collector_context, c"check_collision_point_poly".as_ptr(), Raylib_CheckCollisionPointPoly);
    register_callback(collector_context, c"Raylib::CheckCollisionLines".as_ptr(), Raylib_CheckCollisionLines);
    register_callback(collector_context, c"Raylib::check_collision_lines".as_ptr(), Raylib_CheckCollisionLines);
    register_callback(collector_context, c"raylib::CheckCollisionLines".as_ptr(), Raylib_CheckCollisionLines);
    register_callback(collector_context, c"raylib::check_collision_lines".as_ptr(), Raylib_CheckCollisionLines);
    register_callback(collector_context, c"CheckCollisionLines".as_ptr(), Raylib_CheckCollisionLines);
    register_callback(collector_context, c"check_collision_lines".as_ptr(), Raylib_CheckCollisionLines);
    register_callback(collector_context, c"Raylib::GetCollisionRec".as_ptr(), Raylib_GetCollisionRec);
    register_callback(collector_context, c"Raylib::get_collision_rec".as_ptr(), Raylib_GetCollisionRec);
    register_callback(collector_context, c"raylib::GetCollisionRec".as_ptr(), Raylib_GetCollisionRec);
    register_callback(collector_context, c"raylib::get_collision_rec".as_ptr(), Raylib_GetCollisionRec);
    register_callback(collector_context, c"GetCollisionRec".as_ptr(), Raylib_GetCollisionRec);
    register_callback(collector_context, c"get_collision_rec".as_ptr(), Raylib_GetCollisionRec);
    register_callback(collector_context, c"Raylib::LoadImage".as_ptr(), Raylib_LoadImage);
    register_callback(collector_context, c"Raylib::load_image".as_ptr(), Raylib_LoadImage);
    register_callback(collector_context, c"raylib::LoadImage".as_ptr(), Raylib_LoadImage);
    register_callback(collector_context, c"raylib::load_image".as_ptr(), Raylib_LoadImage);
    register_callback(collector_context, c"LoadImage".as_ptr(), Raylib_LoadImage);
    register_callback(collector_context, c"load_image".as_ptr(), Raylib_LoadImage);
    register_callback(collector_context, c"Raylib::LoadImageRaw".as_ptr(), Raylib_LoadImageRaw);
    register_callback(collector_context, c"Raylib::load_image_raw".as_ptr(), Raylib_LoadImageRaw);
    register_callback(collector_context, c"raylib::LoadImageRaw".as_ptr(), Raylib_LoadImageRaw);
    register_callback(collector_context, c"raylib::load_image_raw".as_ptr(), Raylib_LoadImageRaw);
    register_callback(collector_context, c"LoadImageRaw".as_ptr(), Raylib_LoadImageRaw);
    register_callback(collector_context, c"load_image_raw".as_ptr(), Raylib_LoadImageRaw);
    register_callback(collector_context, c"Raylib::LoadImageAnim".as_ptr(), Raylib_LoadImageAnim);
    register_callback(collector_context, c"Raylib::load_image_anim".as_ptr(), Raylib_LoadImageAnim);
    register_callback(collector_context, c"raylib::LoadImageAnim".as_ptr(), Raylib_LoadImageAnim);
    register_callback(collector_context, c"raylib::load_image_anim".as_ptr(), Raylib_LoadImageAnim);
    register_callback(collector_context, c"LoadImageAnim".as_ptr(), Raylib_LoadImageAnim);
    register_callback(collector_context, c"load_image_anim".as_ptr(), Raylib_LoadImageAnim);
    register_callback(collector_context, c"Raylib::LoadImageAnimFromMemory".as_ptr(), Raylib_LoadImageAnimFromMemory);
    register_callback(collector_context, c"Raylib::load_image_anim_from_memory".as_ptr(), Raylib_LoadImageAnimFromMemory);
    register_callback(collector_context, c"raylib::LoadImageAnimFromMemory".as_ptr(), Raylib_LoadImageAnimFromMemory);
    register_callback(collector_context, c"raylib::load_image_anim_from_memory".as_ptr(), Raylib_LoadImageAnimFromMemory);
    register_callback(collector_context, c"LoadImageAnimFromMemory".as_ptr(), Raylib_LoadImageAnimFromMemory);
    register_callback(collector_context, c"load_image_anim_from_memory".as_ptr(), Raylib_LoadImageAnimFromMemory);
    register_callback(collector_context, c"Raylib::LoadImageFromMemory".as_ptr(), Raylib_LoadImageFromMemory);
    register_callback(collector_context, c"Raylib::load_image_from_memory".as_ptr(), Raylib_LoadImageFromMemory);
    register_callback(collector_context, c"raylib::LoadImageFromMemory".as_ptr(), Raylib_LoadImageFromMemory);
    register_callback(collector_context, c"raylib::load_image_from_memory".as_ptr(), Raylib_LoadImageFromMemory);
    register_callback(collector_context, c"LoadImageFromMemory".as_ptr(), Raylib_LoadImageFromMemory);
    register_callback(collector_context, c"load_image_from_memory".as_ptr(), Raylib_LoadImageFromMemory);
    register_callback(collector_context, c"Raylib::LoadImageFromTexture".as_ptr(), Raylib_LoadImageFromTexture);
    register_callback(collector_context, c"Raylib::load_image_from_texture".as_ptr(), Raylib_LoadImageFromTexture);
    register_callback(collector_context, c"raylib::LoadImageFromTexture".as_ptr(), Raylib_LoadImageFromTexture);
    register_callback(collector_context, c"raylib::load_image_from_texture".as_ptr(), Raylib_LoadImageFromTexture);
    register_callback(collector_context, c"LoadImageFromTexture".as_ptr(), Raylib_LoadImageFromTexture);
    register_callback(collector_context, c"load_image_from_texture".as_ptr(), Raylib_LoadImageFromTexture);
    register_callback(collector_context, c"Raylib::LoadImageFromScreen".as_ptr(), Raylib_LoadImageFromScreen);
    register_callback(collector_context, c"Raylib::load_image_from_screen".as_ptr(), Raylib_LoadImageFromScreen);
    register_callback(collector_context, c"raylib::LoadImageFromScreen".as_ptr(), Raylib_LoadImageFromScreen);
    register_callback(collector_context, c"raylib::load_image_from_screen".as_ptr(), Raylib_LoadImageFromScreen);
    register_callback(collector_context, c"LoadImageFromScreen".as_ptr(), Raylib_LoadImageFromScreen);
    register_callback(collector_context, c"load_image_from_screen".as_ptr(), Raylib_LoadImageFromScreen);
    register_callback(collector_context, c"Raylib::IsImageValid".as_ptr(), Raylib_IsImageValid);
    register_callback(collector_context, c"Raylib::is_image_valid".as_ptr(), Raylib_IsImageValid);
    register_callback(collector_context, c"raylib::IsImageValid".as_ptr(), Raylib_IsImageValid);
    register_callback(collector_context, c"raylib::is_image_valid".as_ptr(), Raylib_IsImageValid);
    register_callback(collector_context, c"IsImageValid".as_ptr(), Raylib_IsImageValid);
    register_callback(collector_context, c"is_image_valid".as_ptr(), Raylib_IsImageValid);
    register_callback(collector_context, c"Raylib::UnloadImage".as_ptr(), Raylib_UnloadImage);
    register_callback(collector_context, c"Raylib::unload_image".as_ptr(), Raylib_UnloadImage);
    register_callback(collector_context, c"raylib::UnloadImage".as_ptr(), Raylib_UnloadImage);
    register_callback(collector_context, c"raylib::unload_image".as_ptr(), Raylib_UnloadImage);
    register_callback(collector_context, c"UnloadImage".as_ptr(), Raylib_UnloadImage);
    register_callback(collector_context, c"unload_image".as_ptr(), Raylib_UnloadImage);
    register_callback(collector_context, c"Raylib::ExportImage".as_ptr(), Raylib_ExportImage);
    register_callback(collector_context, c"Raylib::export_image".as_ptr(), Raylib_ExportImage);
    register_callback(collector_context, c"raylib::ExportImage".as_ptr(), Raylib_ExportImage);
    register_callback(collector_context, c"raylib::export_image".as_ptr(), Raylib_ExportImage);
    register_callback(collector_context, c"ExportImage".as_ptr(), Raylib_ExportImage);
    register_callback(collector_context, c"export_image".as_ptr(), Raylib_ExportImage);
    register_callback(collector_context, c"Raylib::ExportImageToMemory".as_ptr(), Raylib_ExportImageToMemory);
    register_callback(collector_context, c"Raylib::export_image_to_memory".as_ptr(), Raylib_ExportImageToMemory);
    register_callback(collector_context, c"raylib::ExportImageToMemory".as_ptr(), Raylib_ExportImageToMemory);
    register_callback(collector_context, c"raylib::export_image_to_memory".as_ptr(), Raylib_ExportImageToMemory);
    register_callback(collector_context, c"ExportImageToMemory".as_ptr(), Raylib_ExportImageToMemory);
    register_callback(collector_context, c"export_image_to_memory".as_ptr(), Raylib_ExportImageToMemory);
    register_callback(collector_context, c"Raylib::ExportImageAsCode".as_ptr(), Raylib_ExportImageAsCode);
    register_callback(collector_context, c"Raylib::export_image_as_code".as_ptr(), Raylib_ExportImageAsCode);
    register_callback(collector_context, c"raylib::ExportImageAsCode".as_ptr(), Raylib_ExportImageAsCode);
    register_callback(collector_context, c"raylib::export_image_as_code".as_ptr(), Raylib_ExportImageAsCode);
    register_callback(collector_context, c"ExportImageAsCode".as_ptr(), Raylib_ExportImageAsCode);
    register_callback(collector_context, c"export_image_as_code".as_ptr(), Raylib_ExportImageAsCode);
    register_callback(collector_context, c"Raylib::GenImageColor".as_ptr(), Raylib_GenImageColor);
    register_callback(collector_context, c"Raylib::gen_image_color".as_ptr(), Raylib_GenImageColor);
    register_callback(collector_context, c"raylib::GenImageColor".as_ptr(), Raylib_GenImageColor);
    register_callback(collector_context, c"raylib::gen_image_color".as_ptr(), Raylib_GenImageColor);
    register_callback(collector_context, c"GenImageColor".as_ptr(), Raylib_GenImageColor);
    register_callback(collector_context, c"gen_image_color".as_ptr(), Raylib_GenImageColor);
    register_callback(collector_context, c"Raylib::GenImageGradientLinear".as_ptr(), Raylib_GenImageGradientLinear);
    register_callback(collector_context, c"Raylib::gen_image_gradient_linear".as_ptr(), Raylib_GenImageGradientLinear);
    register_callback(collector_context, c"raylib::GenImageGradientLinear".as_ptr(), Raylib_GenImageGradientLinear);
    register_callback(collector_context, c"raylib::gen_image_gradient_linear".as_ptr(), Raylib_GenImageGradientLinear);
    register_callback(collector_context, c"GenImageGradientLinear".as_ptr(), Raylib_GenImageGradientLinear);
    register_callback(collector_context, c"gen_image_gradient_linear".as_ptr(), Raylib_GenImageGradientLinear);
    register_callback(collector_context, c"Raylib::GenImageGradientRadial".as_ptr(), Raylib_GenImageGradientRadial);
    register_callback(collector_context, c"Raylib::gen_image_gradient_radial".as_ptr(), Raylib_GenImageGradientRadial);
    register_callback(collector_context, c"raylib::GenImageGradientRadial".as_ptr(), Raylib_GenImageGradientRadial);
    register_callback(collector_context, c"raylib::gen_image_gradient_radial".as_ptr(), Raylib_GenImageGradientRadial);
    register_callback(collector_context, c"GenImageGradientRadial".as_ptr(), Raylib_GenImageGradientRadial);
    register_callback(collector_context, c"gen_image_gradient_radial".as_ptr(), Raylib_GenImageGradientRadial);
    register_callback(collector_context, c"Raylib::GenImageGradientSquare".as_ptr(), Raylib_GenImageGradientSquare);
    register_callback(collector_context, c"Raylib::gen_image_gradient_square".as_ptr(), Raylib_GenImageGradientSquare);
    register_callback(collector_context, c"raylib::GenImageGradientSquare".as_ptr(), Raylib_GenImageGradientSquare);
    register_callback(collector_context, c"raylib::gen_image_gradient_square".as_ptr(), Raylib_GenImageGradientSquare);
    register_callback(collector_context, c"GenImageGradientSquare".as_ptr(), Raylib_GenImageGradientSquare);
    register_callback(collector_context, c"gen_image_gradient_square".as_ptr(), Raylib_GenImageGradientSquare);
    register_callback(collector_context, c"Raylib::GenImageChecked".as_ptr(), Raylib_GenImageChecked);
    register_callback(collector_context, c"Raylib::gen_image_checked".as_ptr(), Raylib_GenImageChecked);
    register_callback(collector_context, c"raylib::GenImageChecked".as_ptr(), Raylib_GenImageChecked);
    register_callback(collector_context, c"raylib::gen_image_checked".as_ptr(), Raylib_GenImageChecked);
    register_callback(collector_context, c"GenImageChecked".as_ptr(), Raylib_GenImageChecked);
    register_callback(collector_context, c"gen_image_checked".as_ptr(), Raylib_GenImageChecked);
    register_callback(collector_context, c"Raylib::GenImageWhiteNoise".as_ptr(), Raylib_GenImageWhiteNoise);
    register_callback(collector_context, c"Raylib::gen_image_white_noise".as_ptr(), Raylib_GenImageWhiteNoise);
    register_callback(collector_context, c"raylib::GenImageWhiteNoise".as_ptr(), Raylib_GenImageWhiteNoise);
    register_callback(collector_context, c"raylib::gen_image_white_noise".as_ptr(), Raylib_GenImageWhiteNoise);
    register_callback(collector_context, c"GenImageWhiteNoise".as_ptr(), Raylib_GenImageWhiteNoise);
    register_callback(collector_context, c"gen_image_white_noise".as_ptr(), Raylib_GenImageWhiteNoise);
    register_callback(collector_context, c"Raylib::GenImagePerlinNoise".as_ptr(), Raylib_GenImagePerlinNoise);
    register_callback(collector_context, c"Raylib::gen_image_perlin_noise".as_ptr(), Raylib_GenImagePerlinNoise);
    register_callback(collector_context, c"raylib::GenImagePerlinNoise".as_ptr(), Raylib_GenImagePerlinNoise);
    register_callback(collector_context, c"raylib::gen_image_perlin_noise".as_ptr(), Raylib_GenImagePerlinNoise);
    register_callback(collector_context, c"GenImagePerlinNoise".as_ptr(), Raylib_GenImagePerlinNoise);
    register_callback(collector_context, c"gen_image_perlin_noise".as_ptr(), Raylib_GenImagePerlinNoise);
    register_callback(collector_context, c"Raylib::GenImageCellular".as_ptr(), Raylib_GenImageCellular);
    register_callback(collector_context, c"Raylib::gen_image_cellular".as_ptr(), Raylib_GenImageCellular);
    register_callback(collector_context, c"raylib::GenImageCellular".as_ptr(), Raylib_GenImageCellular);
    register_callback(collector_context, c"raylib::gen_image_cellular".as_ptr(), Raylib_GenImageCellular);
    register_callback(collector_context, c"GenImageCellular".as_ptr(), Raylib_GenImageCellular);
    register_callback(collector_context, c"gen_image_cellular".as_ptr(), Raylib_GenImageCellular);
    register_callback(collector_context, c"Raylib::GenImageText".as_ptr(), Raylib_GenImageText);
    register_callback(collector_context, c"Raylib::gen_image_text".as_ptr(), Raylib_GenImageText);
    register_callback(collector_context, c"raylib::GenImageText".as_ptr(), Raylib_GenImageText);
    register_callback(collector_context, c"raylib::gen_image_text".as_ptr(), Raylib_GenImageText);
    register_callback(collector_context, c"GenImageText".as_ptr(), Raylib_GenImageText);
    register_callback(collector_context, c"gen_image_text".as_ptr(), Raylib_GenImageText);
    register_callback(collector_context, c"Raylib::ImageCopy".as_ptr(), Raylib_ImageCopy);
    register_callback(collector_context, c"Raylib::image_copy".as_ptr(), Raylib_ImageCopy);
    register_callback(collector_context, c"raylib::ImageCopy".as_ptr(), Raylib_ImageCopy);
    register_callback(collector_context, c"raylib::image_copy".as_ptr(), Raylib_ImageCopy);
    register_callback(collector_context, c"ImageCopy".as_ptr(), Raylib_ImageCopy);
    register_callback(collector_context, c"image_copy".as_ptr(), Raylib_ImageCopy);
    register_callback(collector_context, c"Raylib::ImageFromImage".as_ptr(), Raylib_ImageFromImage);
    register_callback(collector_context, c"Raylib::image_from_image".as_ptr(), Raylib_ImageFromImage);
    register_callback(collector_context, c"raylib::ImageFromImage".as_ptr(), Raylib_ImageFromImage);
    register_callback(collector_context, c"raylib::image_from_image".as_ptr(), Raylib_ImageFromImage);
    register_callback(collector_context, c"ImageFromImage".as_ptr(), Raylib_ImageFromImage);
    register_callback(collector_context, c"image_from_image".as_ptr(), Raylib_ImageFromImage);
    register_callback(collector_context, c"Raylib::ImageFromChannel".as_ptr(), Raylib_ImageFromChannel);
    register_callback(collector_context, c"Raylib::image_from_channel".as_ptr(), Raylib_ImageFromChannel);
    register_callback(collector_context, c"raylib::ImageFromChannel".as_ptr(), Raylib_ImageFromChannel);
    register_callback(collector_context, c"raylib::image_from_channel".as_ptr(), Raylib_ImageFromChannel);
    register_callback(collector_context, c"ImageFromChannel".as_ptr(), Raylib_ImageFromChannel);
    register_callback(collector_context, c"image_from_channel".as_ptr(), Raylib_ImageFromChannel);
    register_callback(collector_context, c"Raylib::ImageText".as_ptr(), Raylib_ImageText);
    register_callback(collector_context, c"Raylib::image_text".as_ptr(), Raylib_ImageText);
    register_callback(collector_context, c"raylib::ImageText".as_ptr(), Raylib_ImageText);
    register_callback(collector_context, c"raylib::image_text".as_ptr(), Raylib_ImageText);
    register_callback(collector_context, c"ImageText".as_ptr(), Raylib_ImageText);
    register_callback(collector_context, c"image_text".as_ptr(), Raylib_ImageText);
    register_callback(collector_context, c"Raylib::ImageTextEx".as_ptr(), Raylib_ImageTextEx);
    register_callback(collector_context, c"Raylib::image_text_ex".as_ptr(), Raylib_ImageTextEx);
    register_callback(collector_context, c"raylib::ImageTextEx".as_ptr(), Raylib_ImageTextEx);
    register_callback(collector_context, c"raylib::image_text_ex".as_ptr(), Raylib_ImageTextEx);
    register_callback(collector_context, c"ImageTextEx".as_ptr(), Raylib_ImageTextEx);
    register_callback(collector_context, c"image_text_ex".as_ptr(), Raylib_ImageTextEx);
    register_callback(collector_context, c"Raylib::ImageFormat".as_ptr(), Raylib_ImageFormat);
    register_callback(collector_context, c"Raylib::image_format".as_ptr(), Raylib_ImageFormat);
    register_callback(collector_context, c"raylib::ImageFormat".as_ptr(), Raylib_ImageFormat);
    register_callback(collector_context, c"raylib::image_format".as_ptr(), Raylib_ImageFormat);
    register_callback(collector_context, c"ImageFormat".as_ptr(), Raylib_ImageFormat);
    register_callback(collector_context, c"image_format".as_ptr(), Raylib_ImageFormat);
    register_callback(collector_context, c"Raylib::ImageToPOT".as_ptr(), Raylib_ImageToPOT);
    register_callback(collector_context, c"Raylib::image_to_pot".as_ptr(), Raylib_ImageToPOT);
    register_callback(collector_context, c"raylib::ImageToPOT".as_ptr(), Raylib_ImageToPOT);
    register_callback(collector_context, c"raylib::image_to_pot".as_ptr(), Raylib_ImageToPOT);
    register_callback(collector_context, c"ImageToPOT".as_ptr(), Raylib_ImageToPOT);
    register_callback(collector_context, c"image_to_pot".as_ptr(), Raylib_ImageToPOT);
    register_callback(collector_context, c"Raylib::ImageCrop".as_ptr(), Raylib_ImageCrop);
    register_callback(collector_context, c"Raylib::image_crop".as_ptr(), Raylib_ImageCrop);
    register_callback(collector_context, c"raylib::ImageCrop".as_ptr(), Raylib_ImageCrop);
    register_callback(collector_context, c"raylib::image_crop".as_ptr(), Raylib_ImageCrop);
    register_callback(collector_context, c"ImageCrop".as_ptr(), Raylib_ImageCrop);
    register_callback(collector_context, c"image_crop".as_ptr(), Raylib_ImageCrop);
    register_callback(collector_context, c"Raylib::ImageAlphaCrop".as_ptr(), Raylib_ImageAlphaCrop);
    register_callback(collector_context, c"Raylib::image_alpha_crop".as_ptr(), Raylib_ImageAlphaCrop);
    register_callback(collector_context, c"raylib::ImageAlphaCrop".as_ptr(), Raylib_ImageAlphaCrop);
    register_callback(collector_context, c"raylib::image_alpha_crop".as_ptr(), Raylib_ImageAlphaCrop);
    register_callback(collector_context, c"ImageAlphaCrop".as_ptr(), Raylib_ImageAlphaCrop);
    register_callback(collector_context, c"image_alpha_crop".as_ptr(), Raylib_ImageAlphaCrop);
    register_callback(collector_context, c"Raylib::ImageAlphaClear".as_ptr(), Raylib_ImageAlphaClear);
    register_callback(collector_context, c"Raylib::image_alpha_clear".as_ptr(), Raylib_ImageAlphaClear);
    register_callback(collector_context, c"raylib::ImageAlphaClear".as_ptr(), Raylib_ImageAlphaClear);
    register_callback(collector_context, c"raylib::image_alpha_clear".as_ptr(), Raylib_ImageAlphaClear);
    register_callback(collector_context, c"ImageAlphaClear".as_ptr(), Raylib_ImageAlphaClear);
    register_callback(collector_context, c"image_alpha_clear".as_ptr(), Raylib_ImageAlphaClear);
    register_callback(collector_context, c"Raylib::ImageAlphaMask".as_ptr(), Raylib_ImageAlphaMask);
    register_callback(collector_context, c"Raylib::image_alpha_mask".as_ptr(), Raylib_ImageAlphaMask);
    register_callback(collector_context, c"raylib::ImageAlphaMask".as_ptr(), Raylib_ImageAlphaMask);
    register_callback(collector_context, c"raylib::image_alpha_mask".as_ptr(), Raylib_ImageAlphaMask);
    register_callback(collector_context, c"ImageAlphaMask".as_ptr(), Raylib_ImageAlphaMask);
    register_callback(collector_context, c"image_alpha_mask".as_ptr(), Raylib_ImageAlphaMask);
    register_callback(collector_context, c"Raylib::ImageAlphaPremultiply".as_ptr(), Raylib_ImageAlphaPremultiply);
    register_callback(collector_context, c"Raylib::image_alpha_premultiply".as_ptr(), Raylib_ImageAlphaPremultiply);
    register_callback(collector_context, c"raylib::ImageAlphaPremultiply".as_ptr(), Raylib_ImageAlphaPremultiply);
    register_callback(collector_context, c"raylib::image_alpha_premultiply".as_ptr(), Raylib_ImageAlphaPremultiply);
    register_callback(collector_context, c"ImageAlphaPremultiply".as_ptr(), Raylib_ImageAlphaPremultiply);
    register_callback(collector_context, c"image_alpha_premultiply".as_ptr(), Raylib_ImageAlphaPremultiply);
    register_callback(collector_context, c"Raylib::ImageBlurGaussian".as_ptr(), Raylib_ImageBlurGaussian);
    register_callback(collector_context, c"Raylib::image_blur_gaussian".as_ptr(), Raylib_ImageBlurGaussian);
    register_callback(collector_context, c"raylib::ImageBlurGaussian".as_ptr(), Raylib_ImageBlurGaussian);
    register_callback(collector_context, c"raylib::image_blur_gaussian".as_ptr(), Raylib_ImageBlurGaussian);
    register_callback(collector_context, c"ImageBlurGaussian".as_ptr(), Raylib_ImageBlurGaussian);
    register_callback(collector_context, c"image_blur_gaussian".as_ptr(), Raylib_ImageBlurGaussian);
    register_callback(collector_context, c"Raylib::ImageKernelConvolution".as_ptr(), Raylib_ImageKernelConvolution);
    register_callback(collector_context, c"Raylib::image_kernel_convolution".as_ptr(), Raylib_ImageKernelConvolution);
    register_callback(collector_context, c"raylib::ImageKernelConvolution".as_ptr(), Raylib_ImageKernelConvolution);
    register_callback(collector_context, c"raylib::image_kernel_convolution".as_ptr(), Raylib_ImageKernelConvolution);
    register_callback(collector_context, c"ImageKernelConvolution".as_ptr(), Raylib_ImageKernelConvolution);
    register_callback(collector_context, c"image_kernel_convolution".as_ptr(), Raylib_ImageKernelConvolution);
    register_callback(collector_context, c"Raylib::ImageResize".as_ptr(), Raylib_ImageResize);
    register_callback(collector_context, c"Raylib::image_resize".as_ptr(), Raylib_ImageResize);
    register_callback(collector_context, c"raylib::ImageResize".as_ptr(), Raylib_ImageResize);
    register_callback(collector_context, c"raylib::image_resize".as_ptr(), Raylib_ImageResize);
    register_callback(collector_context, c"ImageResize".as_ptr(), Raylib_ImageResize);
    register_callback(collector_context, c"image_resize".as_ptr(), Raylib_ImageResize);
    register_callback(collector_context, c"Raylib::ImageResizeNN".as_ptr(), Raylib_ImageResizeNN);
    register_callback(collector_context, c"Raylib::image_resize_nn".as_ptr(), Raylib_ImageResizeNN);
    register_callback(collector_context, c"raylib::ImageResizeNN".as_ptr(), Raylib_ImageResizeNN);
    register_callback(collector_context, c"raylib::image_resize_nn".as_ptr(), Raylib_ImageResizeNN);
    register_callback(collector_context, c"ImageResizeNN".as_ptr(), Raylib_ImageResizeNN);
    register_callback(collector_context, c"image_resize_nn".as_ptr(), Raylib_ImageResizeNN);
    register_callback(collector_context, c"Raylib::ImageResizeCanvas".as_ptr(), Raylib_ImageResizeCanvas);
    register_callback(collector_context, c"Raylib::image_resize_canvas".as_ptr(), Raylib_ImageResizeCanvas);
    register_callback(collector_context, c"raylib::ImageResizeCanvas".as_ptr(), Raylib_ImageResizeCanvas);
    register_callback(collector_context, c"raylib::image_resize_canvas".as_ptr(), Raylib_ImageResizeCanvas);
    register_callback(collector_context, c"ImageResizeCanvas".as_ptr(), Raylib_ImageResizeCanvas);
    register_callback(collector_context, c"image_resize_canvas".as_ptr(), Raylib_ImageResizeCanvas);
    register_callback(collector_context, c"Raylib::ImageMipmaps".as_ptr(), Raylib_ImageMipmaps);
    register_callback(collector_context, c"Raylib::image_mipmaps".as_ptr(), Raylib_ImageMipmaps);
    register_callback(collector_context, c"raylib::ImageMipmaps".as_ptr(), Raylib_ImageMipmaps);
    register_callback(collector_context, c"raylib::image_mipmaps".as_ptr(), Raylib_ImageMipmaps);
    register_callback(collector_context, c"ImageMipmaps".as_ptr(), Raylib_ImageMipmaps);
    register_callback(collector_context, c"image_mipmaps".as_ptr(), Raylib_ImageMipmaps);
    register_callback(collector_context, c"Raylib::ImageDither".as_ptr(), Raylib_ImageDither);
    register_callback(collector_context, c"Raylib::image_dither".as_ptr(), Raylib_ImageDither);
    register_callback(collector_context, c"raylib::ImageDither".as_ptr(), Raylib_ImageDither);
    register_callback(collector_context, c"raylib::image_dither".as_ptr(), Raylib_ImageDither);
    register_callback(collector_context, c"ImageDither".as_ptr(), Raylib_ImageDither);
    register_callback(collector_context, c"image_dither".as_ptr(), Raylib_ImageDither);
    register_callback(collector_context, c"Raylib::ImageFlipVertical".as_ptr(), Raylib_ImageFlipVertical);
    register_callback(collector_context, c"Raylib::image_flip_vertical".as_ptr(), Raylib_ImageFlipVertical);
    register_callback(collector_context, c"raylib::ImageFlipVertical".as_ptr(), Raylib_ImageFlipVertical);
    register_callback(collector_context, c"raylib::image_flip_vertical".as_ptr(), Raylib_ImageFlipVertical);
    register_callback(collector_context, c"ImageFlipVertical".as_ptr(), Raylib_ImageFlipVertical);
    register_callback(collector_context, c"image_flip_vertical".as_ptr(), Raylib_ImageFlipVertical);
    register_callback(collector_context, c"Raylib::ImageFlipHorizontal".as_ptr(), Raylib_ImageFlipHorizontal);
    register_callback(collector_context, c"Raylib::image_flip_horizontal".as_ptr(), Raylib_ImageFlipHorizontal);
    register_callback(collector_context, c"raylib::ImageFlipHorizontal".as_ptr(), Raylib_ImageFlipHorizontal);
    register_callback(collector_context, c"raylib::image_flip_horizontal".as_ptr(), Raylib_ImageFlipHorizontal);
    register_callback(collector_context, c"ImageFlipHorizontal".as_ptr(), Raylib_ImageFlipHorizontal);
    register_callback(collector_context, c"image_flip_horizontal".as_ptr(), Raylib_ImageFlipHorizontal);
    register_callback(collector_context, c"Raylib::ImageRotate".as_ptr(), Raylib_ImageRotate);
    register_callback(collector_context, c"Raylib::image_rotate".as_ptr(), Raylib_ImageRotate);
    register_callback(collector_context, c"raylib::ImageRotate".as_ptr(), Raylib_ImageRotate);
    register_callback(collector_context, c"raylib::image_rotate".as_ptr(), Raylib_ImageRotate);
    register_callback(collector_context, c"ImageRotate".as_ptr(), Raylib_ImageRotate);
    register_callback(collector_context, c"image_rotate".as_ptr(), Raylib_ImageRotate);
    register_callback(collector_context, c"Raylib::ImageRotateCW".as_ptr(), Raylib_ImageRotateCW);
    register_callback(collector_context, c"Raylib::image_rotate_cw".as_ptr(), Raylib_ImageRotateCW);
    register_callback(collector_context, c"raylib::ImageRotateCW".as_ptr(), Raylib_ImageRotateCW);
    register_callback(collector_context, c"raylib::image_rotate_cw".as_ptr(), Raylib_ImageRotateCW);
    register_callback(collector_context, c"ImageRotateCW".as_ptr(), Raylib_ImageRotateCW);
    register_callback(collector_context, c"image_rotate_cw".as_ptr(), Raylib_ImageRotateCW);
    register_callback(collector_context, c"Raylib::ImageRotateCCW".as_ptr(), Raylib_ImageRotateCCW);
    register_callback(collector_context, c"Raylib::image_rotate_ccw".as_ptr(), Raylib_ImageRotateCCW);
    register_callback(collector_context, c"raylib::ImageRotateCCW".as_ptr(), Raylib_ImageRotateCCW);
    register_callback(collector_context, c"raylib::image_rotate_ccw".as_ptr(), Raylib_ImageRotateCCW);
    register_callback(collector_context, c"ImageRotateCCW".as_ptr(), Raylib_ImageRotateCCW);
    register_callback(collector_context, c"image_rotate_ccw".as_ptr(), Raylib_ImageRotateCCW);
    register_callback(collector_context, c"Raylib::ImageColorTint".as_ptr(), Raylib_ImageColorTint);
    register_callback(collector_context, c"Raylib::image_color_tint".as_ptr(), Raylib_ImageColorTint);
    register_callback(collector_context, c"raylib::ImageColorTint".as_ptr(), Raylib_ImageColorTint);
    register_callback(collector_context, c"raylib::image_color_tint".as_ptr(), Raylib_ImageColorTint);
    register_callback(collector_context, c"ImageColorTint".as_ptr(), Raylib_ImageColorTint);
    register_callback(collector_context, c"image_color_tint".as_ptr(), Raylib_ImageColorTint);
    register_callback(collector_context, c"Raylib::ImageColorInvert".as_ptr(), Raylib_ImageColorInvert);
    register_callback(collector_context, c"Raylib::image_color_invert".as_ptr(), Raylib_ImageColorInvert);
    register_callback(collector_context, c"raylib::ImageColorInvert".as_ptr(), Raylib_ImageColorInvert);
    register_callback(collector_context, c"raylib::image_color_invert".as_ptr(), Raylib_ImageColorInvert);
    register_callback(collector_context, c"ImageColorInvert".as_ptr(), Raylib_ImageColorInvert);
    register_callback(collector_context, c"image_color_invert".as_ptr(), Raylib_ImageColorInvert);
    register_callback(collector_context, c"Raylib::ImageColorGrayscale".as_ptr(), Raylib_ImageColorGrayscale);
    register_callback(collector_context, c"Raylib::image_color_grayscale".as_ptr(), Raylib_ImageColorGrayscale);
    register_callback(collector_context, c"raylib::ImageColorGrayscale".as_ptr(), Raylib_ImageColorGrayscale);
    register_callback(collector_context, c"raylib::image_color_grayscale".as_ptr(), Raylib_ImageColorGrayscale);
    register_callback(collector_context, c"ImageColorGrayscale".as_ptr(), Raylib_ImageColorGrayscale);
    register_callback(collector_context, c"image_color_grayscale".as_ptr(), Raylib_ImageColorGrayscale);
    register_callback(collector_context, c"Raylib::ImageColorContrast".as_ptr(), Raylib_ImageColorContrast);
    register_callback(collector_context, c"Raylib::image_color_contrast".as_ptr(), Raylib_ImageColorContrast);
    register_callback(collector_context, c"raylib::ImageColorContrast".as_ptr(), Raylib_ImageColorContrast);
    register_callback(collector_context, c"raylib::image_color_contrast".as_ptr(), Raylib_ImageColorContrast);
    register_callback(collector_context, c"ImageColorContrast".as_ptr(), Raylib_ImageColorContrast);
    register_callback(collector_context, c"image_color_contrast".as_ptr(), Raylib_ImageColorContrast);
    register_callback(collector_context, c"Raylib::ImageColorBrightness".as_ptr(), Raylib_ImageColorBrightness);
    register_callback(collector_context, c"Raylib::image_color_brightness".as_ptr(), Raylib_ImageColorBrightness);
    register_callback(collector_context, c"raylib::ImageColorBrightness".as_ptr(), Raylib_ImageColorBrightness);
    register_callback(collector_context, c"raylib::image_color_brightness".as_ptr(), Raylib_ImageColorBrightness);
    register_callback(collector_context, c"ImageColorBrightness".as_ptr(), Raylib_ImageColorBrightness);
    register_callback(collector_context, c"image_color_brightness".as_ptr(), Raylib_ImageColorBrightness);
    register_callback(collector_context, c"Raylib::ImageColorReplace".as_ptr(), Raylib_ImageColorReplace);
    register_callback(collector_context, c"Raylib::image_color_replace".as_ptr(), Raylib_ImageColorReplace);
    register_callback(collector_context, c"raylib::ImageColorReplace".as_ptr(), Raylib_ImageColorReplace);
    register_callback(collector_context, c"raylib::image_color_replace".as_ptr(), Raylib_ImageColorReplace);
    register_callback(collector_context, c"ImageColorReplace".as_ptr(), Raylib_ImageColorReplace);
    register_callback(collector_context, c"image_color_replace".as_ptr(), Raylib_ImageColorReplace);
    register_callback(collector_context, c"Raylib::LoadImageColors".as_ptr(), Raylib_LoadImageColors);
    register_callback(collector_context, c"Raylib::load_image_colors".as_ptr(), Raylib_LoadImageColors);
    register_callback(collector_context, c"raylib::LoadImageColors".as_ptr(), Raylib_LoadImageColors);
    register_callback(collector_context, c"raylib::load_image_colors".as_ptr(), Raylib_LoadImageColors);
    register_callback(collector_context, c"LoadImageColors".as_ptr(), Raylib_LoadImageColors);
    register_callback(collector_context, c"load_image_colors".as_ptr(), Raylib_LoadImageColors);
    register_callback(collector_context, c"Raylib::LoadImagePalette".as_ptr(), Raylib_LoadImagePalette);
    register_callback(collector_context, c"Raylib::load_image_palette".as_ptr(), Raylib_LoadImagePalette);
    register_callback(collector_context, c"raylib::LoadImagePalette".as_ptr(), Raylib_LoadImagePalette);
    register_callback(collector_context, c"raylib::load_image_palette".as_ptr(), Raylib_LoadImagePalette);
    register_callback(collector_context, c"LoadImagePalette".as_ptr(), Raylib_LoadImagePalette);
    register_callback(collector_context, c"load_image_palette".as_ptr(), Raylib_LoadImagePalette);
    register_callback(collector_context, c"Raylib::UnloadImageColors".as_ptr(), Raylib_UnloadImageColors);
    register_callback(collector_context, c"Raylib::unload_image_colors".as_ptr(), Raylib_UnloadImageColors);
    register_callback(collector_context, c"raylib::UnloadImageColors".as_ptr(), Raylib_UnloadImageColors);
    register_callback(collector_context, c"raylib::unload_image_colors".as_ptr(), Raylib_UnloadImageColors);
    register_callback(collector_context, c"UnloadImageColors".as_ptr(), Raylib_UnloadImageColors);
    register_callback(collector_context, c"unload_image_colors".as_ptr(), Raylib_UnloadImageColors);
    register_callback(collector_context, c"Raylib::UnloadImagePalette".as_ptr(), Raylib_UnloadImagePalette);
    register_callback(collector_context, c"Raylib::unload_image_palette".as_ptr(), Raylib_UnloadImagePalette);
    register_callback(collector_context, c"raylib::UnloadImagePalette".as_ptr(), Raylib_UnloadImagePalette);
    register_callback(collector_context, c"raylib::unload_image_palette".as_ptr(), Raylib_UnloadImagePalette);
    register_callback(collector_context, c"UnloadImagePalette".as_ptr(), Raylib_UnloadImagePalette);
    register_callback(collector_context, c"unload_image_palette".as_ptr(), Raylib_UnloadImagePalette);
    register_callback(collector_context, c"Raylib::GetImageAlphaBorder".as_ptr(), Raylib_GetImageAlphaBorder);
    register_callback(collector_context, c"Raylib::get_image_alpha_border".as_ptr(), Raylib_GetImageAlphaBorder);
    register_callback(collector_context, c"raylib::GetImageAlphaBorder".as_ptr(), Raylib_GetImageAlphaBorder);
    register_callback(collector_context, c"raylib::get_image_alpha_border".as_ptr(), Raylib_GetImageAlphaBorder);
    register_callback(collector_context, c"GetImageAlphaBorder".as_ptr(), Raylib_GetImageAlphaBorder);
    register_callback(collector_context, c"get_image_alpha_border".as_ptr(), Raylib_GetImageAlphaBorder);
    register_callback(collector_context, c"Raylib::GetImageColor".as_ptr(), Raylib_GetImageColor);
    register_callback(collector_context, c"Raylib::get_image_color".as_ptr(), Raylib_GetImageColor);
    register_callback(collector_context, c"raylib::GetImageColor".as_ptr(), Raylib_GetImageColor);
    register_callback(collector_context, c"raylib::get_image_color".as_ptr(), Raylib_GetImageColor);
    register_callback(collector_context, c"GetImageColor".as_ptr(), Raylib_GetImageColor);
    register_callback(collector_context, c"get_image_color".as_ptr(), Raylib_GetImageColor);
    register_callback(collector_context, c"Raylib::ImageClearBackground".as_ptr(), Raylib_ImageClearBackground);
    register_callback(collector_context, c"Raylib::image_clear_background".as_ptr(), Raylib_ImageClearBackground);
    register_callback(collector_context, c"raylib::ImageClearBackground".as_ptr(), Raylib_ImageClearBackground);
    register_callback(collector_context, c"raylib::image_clear_background".as_ptr(), Raylib_ImageClearBackground);
    register_callback(collector_context, c"ImageClearBackground".as_ptr(), Raylib_ImageClearBackground);
    register_callback(collector_context, c"image_clear_background".as_ptr(), Raylib_ImageClearBackground);
    register_callback(collector_context, c"Raylib::ImageDrawPixel".as_ptr(), Raylib_ImageDrawPixel);
    register_callback(collector_context, c"Raylib::image_draw_pixel".as_ptr(), Raylib_ImageDrawPixel);
    register_callback(collector_context, c"raylib::ImageDrawPixel".as_ptr(), Raylib_ImageDrawPixel);
    register_callback(collector_context, c"raylib::image_draw_pixel".as_ptr(), Raylib_ImageDrawPixel);
    register_callback(collector_context, c"ImageDrawPixel".as_ptr(), Raylib_ImageDrawPixel);
    register_callback(collector_context, c"image_draw_pixel".as_ptr(), Raylib_ImageDrawPixel);
    register_callback(collector_context, c"Raylib::ImageDrawPixelV".as_ptr(), Raylib_ImageDrawPixelV);
    register_callback(collector_context, c"Raylib::image_draw_pixel_v".as_ptr(), Raylib_ImageDrawPixelV);
    register_callback(collector_context, c"raylib::ImageDrawPixelV".as_ptr(), Raylib_ImageDrawPixelV);
    register_callback(collector_context, c"raylib::image_draw_pixel_v".as_ptr(), Raylib_ImageDrawPixelV);
    register_callback(collector_context, c"ImageDrawPixelV".as_ptr(), Raylib_ImageDrawPixelV);
    register_callback(collector_context, c"image_draw_pixel_v".as_ptr(), Raylib_ImageDrawPixelV);
    register_callback(collector_context, c"Raylib::ImageDrawLine".as_ptr(), Raylib_ImageDrawLine);
    register_callback(collector_context, c"Raylib::image_draw_line".as_ptr(), Raylib_ImageDrawLine);
    register_callback(collector_context, c"raylib::ImageDrawLine".as_ptr(), Raylib_ImageDrawLine);
    register_callback(collector_context, c"raylib::image_draw_line".as_ptr(), Raylib_ImageDrawLine);
    register_callback(collector_context, c"ImageDrawLine".as_ptr(), Raylib_ImageDrawLine);
    register_callback(collector_context, c"image_draw_line".as_ptr(), Raylib_ImageDrawLine);
    register_callback(collector_context, c"Raylib::ImageDrawLineV".as_ptr(), Raylib_ImageDrawLineV);
    register_callback(collector_context, c"Raylib::image_draw_line_v".as_ptr(), Raylib_ImageDrawLineV);
    register_callback(collector_context, c"raylib::ImageDrawLineV".as_ptr(), Raylib_ImageDrawLineV);
    register_callback(collector_context, c"raylib::image_draw_line_v".as_ptr(), Raylib_ImageDrawLineV);
    register_callback(collector_context, c"ImageDrawLineV".as_ptr(), Raylib_ImageDrawLineV);
    register_callback(collector_context, c"image_draw_line_v".as_ptr(), Raylib_ImageDrawLineV);
    register_callback(collector_context, c"Raylib::ImageDrawLineEx".as_ptr(), Raylib_ImageDrawLineEx);
    register_callback(collector_context, c"Raylib::image_draw_line_ex".as_ptr(), Raylib_ImageDrawLineEx);
    register_callback(collector_context, c"raylib::ImageDrawLineEx".as_ptr(), Raylib_ImageDrawLineEx);
    register_callback(collector_context, c"raylib::image_draw_line_ex".as_ptr(), Raylib_ImageDrawLineEx);
    register_callback(collector_context, c"ImageDrawLineEx".as_ptr(), Raylib_ImageDrawLineEx);
    register_callback(collector_context, c"image_draw_line_ex".as_ptr(), Raylib_ImageDrawLineEx);
    register_callback(collector_context, c"Raylib::ImageDrawLineStrip".as_ptr(), Raylib_ImageDrawLineStrip);
    register_callback(collector_context, c"Raylib::image_draw_line_strip".as_ptr(), Raylib_ImageDrawLineStrip);
    register_callback(collector_context, c"raylib::ImageDrawLineStrip".as_ptr(), Raylib_ImageDrawLineStrip);
    register_callback(collector_context, c"raylib::image_draw_line_strip".as_ptr(), Raylib_ImageDrawLineStrip);
    register_callback(collector_context, c"ImageDrawLineStrip".as_ptr(), Raylib_ImageDrawLineStrip);
    register_callback(collector_context, c"image_draw_line_strip".as_ptr(), Raylib_ImageDrawLineStrip);
    register_callback(collector_context, c"Raylib::ImageDrawTriangle".as_ptr(), Raylib_ImageDrawTriangle);
    register_callback(collector_context, c"Raylib::image_draw_triangle".as_ptr(), Raylib_ImageDrawTriangle);
    register_callback(collector_context, c"raylib::ImageDrawTriangle".as_ptr(), Raylib_ImageDrawTriangle);
    register_callback(collector_context, c"raylib::image_draw_triangle".as_ptr(), Raylib_ImageDrawTriangle);
    register_callback(collector_context, c"ImageDrawTriangle".as_ptr(), Raylib_ImageDrawTriangle);
    register_callback(collector_context, c"image_draw_triangle".as_ptr(), Raylib_ImageDrawTriangle);
    register_callback(collector_context, c"Raylib::ImageDrawTriangleGradient".as_ptr(), Raylib_ImageDrawTriangleGradient);
    register_callback(collector_context, c"Raylib::image_draw_triangle_gradient".as_ptr(), Raylib_ImageDrawTriangleGradient);
    register_callback(collector_context, c"raylib::ImageDrawTriangleGradient".as_ptr(), Raylib_ImageDrawTriangleGradient);
    register_callback(collector_context, c"raylib::image_draw_triangle_gradient".as_ptr(), Raylib_ImageDrawTriangleGradient);
    register_callback(collector_context, c"ImageDrawTriangleGradient".as_ptr(), Raylib_ImageDrawTriangleGradient);
    register_callback(collector_context, c"image_draw_triangle_gradient".as_ptr(), Raylib_ImageDrawTriangleGradient);
    register_callback(collector_context, c"Raylib::ImageDrawTriangleLines".as_ptr(), Raylib_ImageDrawTriangleLines);
    register_callback(collector_context, c"Raylib::image_draw_triangle_lines".as_ptr(), Raylib_ImageDrawTriangleLines);
    register_callback(collector_context, c"raylib::ImageDrawTriangleLines".as_ptr(), Raylib_ImageDrawTriangleLines);
    register_callback(collector_context, c"raylib::image_draw_triangle_lines".as_ptr(), Raylib_ImageDrawTriangleLines);
    register_callback(collector_context, c"ImageDrawTriangleLines".as_ptr(), Raylib_ImageDrawTriangleLines);
    register_callback(collector_context, c"image_draw_triangle_lines".as_ptr(), Raylib_ImageDrawTriangleLines);
    register_callback(collector_context, c"Raylib::ImageDrawTriangleFan".as_ptr(), Raylib_ImageDrawTriangleFan);
    register_callback(collector_context, c"Raylib::image_draw_triangle_fan".as_ptr(), Raylib_ImageDrawTriangleFan);
    register_callback(collector_context, c"raylib::ImageDrawTriangleFan".as_ptr(), Raylib_ImageDrawTriangleFan);
    register_callback(collector_context, c"raylib::image_draw_triangle_fan".as_ptr(), Raylib_ImageDrawTriangleFan);
    register_callback(collector_context, c"ImageDrawTriangleFan".as_ptr(), Raylib_ImageDrawTriangleFan);
    register_callback(collector_context, c"image_draw_triangle_fan".as_ptr(), Raylib_ImageDrawTriangleFan);
    register_callback(collector_context, c"Raylib::ImageDrawTriangleStrip".as_ptr(), Raylib_ImageDrawTriangleStrip);
    register_callback(collector_context, c"Raylib::image_draw_triangle_strip".as_ptr(), Raylib_ImageDrawTriangleStrip);
    register_callback(collector_context, c"raylib::ImageDrawTriangleStrip".as_ptr(), Raylib_ImageDrawTriangleStrip);
    register_callback(collector_context, c"raylib::image_draw_triangle_strip".as_ptr(), Raylib_ImageDrawTriangleStrip);
    register_callback(collector_context, c"ImageDrawTriangleStrip".as_ptr(), Raylib_ImageDrawTriangleStrip);
    register_callback(collector_context, c"image_draw_triangle_strip".as_ptr(), Raylib_ImageDrawTriangleStrip);
    register_callback(collector_context, c"Raylib::ImageDrawRectangle".as_ptr(), Raylib_ImageDrawRectangle);
    register_callback(collector_context, c"Raylib::image_draw_rectangle".as_ptr(), Raylib_ImageDrawRectangle);
    register_callback(collector_context, c"raylib::ImageDrawRectangle".as_ptr(), Raylib_ImageDrawRectangle);
    register_callback(collector_context, c"raylib::image_draw_rectangle".as_ptr(), Raylib_ImageDrawRectangle);
    register_callback(collector_context, c"ImageDrawRectangle".as_ptr(), Raylib_ImageDrawRectangle);
    register_callback(collector_context, c"image_draw_rectangle".as_ptr(), Raylib_ImageDrawRectangle);
    register_callback(collector_context, c"Raylib::ImageDrawRectangleV".as_ptr(), Raylib_ImageDrawRectangleV);
    register_callback(collector_context, c"Raylib::image_draw_rectangle_v".as_ptr(), Raylib_ImageDrawRectangleV);
    register_callback(collector_context, c"raylib::ImageDrawRectangleV".as_ptr(), Raylib_ImageDrawRectangleV);
    register_callback(collector_context, c"raylib::image_draw_rectangle_v".as_ptr(), Raylib_ImageDrawRectangleV);
    register_callback(collector_context, c"ImageDrawRectangleV".as_ptr(), Raylib_ImageDrawRectangleV);
    register_callback(collector_context, c"image_draw_rectangle_v".as_ptr(), Raylib_ImageDrawRectangleV);
    register_callback(collector_context, c"Raylib::ImageDrawRectangleRec".as_ptr(), Raylib_ImageDrawRectangleRec);
    register_callback(collector_context, c"Raylib::image_draw_rectangle_rec".as_ptr(), Raylib_ImageDrawRectangleRec);
    register_callback(collector_context, c"raylib::ImageDrawRectangleRec".as_ptr(), Raylib_ImageDrawRectangleRec);
    register_callback(collector_context, c"raylib::image_draw_rectangle_rec".as_ptr(), Raylib_ImageDrawRectangleRec);
    register_callback(collector_context, c"ImageDrawRectangleRec".as_ptr(), Raylib_ImageDrawRectangleRec);
    register_callback(collector_context, c"image_draw_rectangle_rec".as_ptr(), Raylib_ImageDrawRectangleRec);
    register_callback(collector_context, c"Raylib::ImageDrawRectanglePro".as_ptr(), Raylib_ImageDrawRectanglePro);
    register_callback(collector_context, c"Raylib::image_draw_rectangle_pro".as_ptr(), Raylib_ImageDrawRectanglePro);
    register_callback(collector_context, c"raylib::ImageDrawRectanglePro".as_ptr(), Raylib_ImageDrawRectanglePro);
    register_callback(collector_context, c"raylib::image_draw_rectangle_pro".as_ptr(), Raylib_ImageDrawRectanglePro);
    register_callback(collector_context, c"ImageDrawRectanglePro".as_ptr(), Raylib_ImageDrawRectanglePro);
    register_callback(collector_context, c"image_draw_rectangle_pro".as_ptr(), Raylib_ImageDrawRectanglePro);
    register_callback(collector_context, c"Raylib::ImageDrawRectangleLines".as_ptr(), Raylib_ImageDrawRectangleLines);
    register_callback(collector_context, c"Raylib::image_draw_rectangle_lines".as_ptr(), Raylib_ImageDrawRectangleLines);
    register_callback(collector_context, c"raylib::ImageDrawRectangleLines".as_ptr(), Raylib_ImageDrawRectangleLines);
    register_callback(collector_context, c"raylib::image_draw_rectangle_lines".as_ptr(), Raylib_ImageDrawRectangleLines);
    register_callback(collector_context, c"ImageDrawRectangleLines".as_ptr(), Raylib_ImageDrawRectangleLines);
    register_callback(collector_context, c"image_draw_rectangle_lines".as_ptr(), Raylib_ImageDrawRectangleLines);
    register_callback(collector_context, c"Raylib::ImageDrawRectangleLinesEx".as_ptr(), Raylib_ImageDrawRectangleLinesEx);
    register_callback(collector_context, c"Raylib::image_draw_rectangle_lines_ex".as_ptr(), Raylib_ImageDrawRectangleLinesEx);
    register_callback(collector_context, c"raylib::ImageDrawRectangleLinesEx".as_ptr(), Raylib_ImageDrawRectangleLinesEx);
    register_callback(collector_context, c"raylib::image_draw_rectangle_lines_ex".as_ptr(), Raylib_ImageDrawRectangleLinesEx);
    register_callback(collector_context, c"ImageDrawRectangleLinesEx".as_ptr(), Raylib_ImageDrawRectangleLinesEx);
    register_callback(collector_context, c"image_draw_rectangle_lines_ex".as_ptr(), Raylib_ImageDrawRectangleLinesEx);
    register_callback(collector_context, c"Raylib::ImageDrawRectangleGradientEx".as_ptr(), Raylib_ImageDrawRectangleGradientEx);
    register_callback(collector_context, c"Raylib::image_draw_rectangle_gradient_ex".as_ptr(), Raylib_ImageDrawRectangleGradientEx);
    register_callback(collector_context, c"raylib::ImageDrawRectangleGradientEx".as_ptr(), Raylib_ImageDrawRectangleGradientEx);
    register_callback(collector_context, c"raylib::image_draw_rectangle_gradient_ex".as_ptr(), Raylib_ImageDrawRectangleGradientEx);
    register_callback(collector_context, c"ImageDrawRectangleGradientEx".as_ptr(), Raylib_ImageDrawRectangleGradientEx);
    register_callback(collector_context, c"image_draw_rectangle_gradient_ex".as_ptr(), Raylib_ImageDrawRectangleGradientEx);
    register_callback(collector_context, c"Raylib::ImageDrawCircle".as_ptr(), Raylib_ImageDrawCircle);
    register_callback(collector_context, c"Raylib::image_draw_circle".as_ptr(), Raylib_ImageDrawCircle);
    register_callback(collector_context, c"raylib::ImageDrawCircle".as_ptr(), Raylib_ImageDrawCircle);
    register_callback(collector_context, c"raylib::image_draw_circle".as_ptr(), Raylib_ImageDrawCircle);
    register_callback(collector_context, c"ImageDrawCircle".as_ptr(), Raylib_ImageDrawCircle);
    register_callback(collector_context, c"image_draw_circle".as_ptr(), Raylib_ImageDrawCircle);
    register_callback(collector_context, c"Raylib::ImageDrawCircleV".as_ptr(), Raylib_ImageDrawCircleV);
    register_callback(collector_context, c"Raylib::image_draw_circle_v".as_ptr(), Raylib_ImageDrawCircleV);
    register_callback(collector_context, c"raylib::ImageDrawCircleV".as_ptr(), Raylib_ImageDrawCircleV);
    register_callback(collector_context, c"raylib::image_draw_circle_v".as_ptr(), Raylib_ImageDrawCircleV);
    register_callback(collector_context, c"ImageDrawCircleV".as_ptr(), Raylib_ImageDrawCircleV);
    register_callback(collector_context, c"image_draw_circle_v".as_ptr(), Raylib_ImageDrawCircleV);
    register_callback(collector_context, c"Raylib::ImageDrawCircleLines".as_ptr(), Raylib_ImageDrawCircleLines);
    register_callback(collector_context, c"Raylib::image_draw_circle_lines".as_ptr(), Raylib_ImageDrawCircleLines);
    register_callback(collector_context, c"raylib::ImageDrawCircleLines".as_ptr(), Raylib_ImageDrawCircleLines);
    register_callback(collector_context, c"raylib::image_draw_circle_lines".as_ptr(), Raylib_ImageDrawCircleLines);
    register_callback(collector_context, c"ImageDrawCircleLines".as_ptr(), Raylib_ImageDrawCircleLines);
    register_callback(collector_context, c"image_draw_circle_lines".as_ptr(), Raylib_ImageDrawCircleLines);
    register_callback(collector_context, c"Raylib::ImageDrawCircleLinesV".as_ptr(), Raylib_ImageDrawCircleLinesV);
    register_callback(collector_context, c"Raylib::image_draw_circle_lines_v".as_ptr(), Raylib_ImageDrawCircleLinesV);
    register_callback(collector_context, c"raylib::ImageDrawCircleLinesV".as_ptr(), Raylib_ImageDrawCircleLinesV);
    register_callback(collector_context, c"raylib::image_draw_circle_lines_v".as_ptr(), Raylib_ImageDrawCircleLinesV);
    register_callback(collector_context, c"ImageDrawCircleLinesV".as_ptr(), Raylib_ImageDrawCircleLinesV);
    register_callback(collector_context, c"image_draw_circle_lines_v".as_ptr(), Raylib_ImageDrawCircleLinesV);
    register_callback(collector_context, c"Raylib::ImageDrawCircleGradient".as_ptr(), Raylib_ImageDrawCircleGradient);
    register_callback(collector_context, c"Raylib::image_draw_circle_gradient".as_ptr(), Raylib_ImageDrawCircleGradient);
    register_callback(collector_context, c"raylib::ImageDrawCircleGradient".as_ptr(), Raylib_ImageDrawCircleGradient);
    register_callback(collector_context, c"raylib::image_draw_circle_gradient".as_ptr(), Raylib_ImageDrawCircleGradient);
    register_callback(collector_context, c"ImageDrawCircleGradient".as_ptr(), Raylib_ImageDrawCircleGradient);
    register_callback(collector_context, c"image_draw_circle_gradient".as_ptr(), Raylib_ImageDrawCircleGradient);
    register_callback(collector_context, c"Raylib::ImageDrawImage".as_ptr(), Raylib_ImageDrawImage);
    register_callback(collector_context, c"Raylib::image_draw_image".as_ptr(), Raylib_ImageDrawImage);
    register_callback(collector_context, c"raylib::ImageDrawImage".as_ptr(), Raylib_ImageDrawImage);
    register_callback(collector_context, c"raylib::image_draw_image".as_ptr(), Raylib_ImageDrawImage);
    register_callback(collector_context, c"ImageDrawImage".as_ptr(), Raylib_ImageDrawImage);
    register_callback(collector_context, c"image_draw_image".as_ptr(), Raylib_ImageDrawImage);
    register_callback(collector_context, c"Raylib::ImageDrawImageEx".as_ptr(), Raylib_ImageDrawImageEx);
    register_callback(collector_context, c"Raylib::image_draw_image_ex".as_ptr(), Raylib_ImageDrawImageEx);
    register_callback(collector_context, c"raylib::ImageDrawImageEx".as_ptr(), Raylib_ImageDrawImageEx);
    register_callback(collector_context, c"raylib::image_draw_image_ex".as_ptr(), Raylib_ImageDrawImageEx);
    register_callback(collector_context, c"ImageDrawImageEx".as_ptr(), Raylib_ImageDrawImageEx);
    register_callback(collector_context, c"image_draw_image_ex".as_ptr(), Raylib_ImageDrawImageEx);
    register_callback(collector_context, c"Raylib::ImageDrawImageRec".as_ptr(), Raylib_ImageDrawImageRec);
    register_callback(collector_context, c"Raylib::image_draw_image_rec".as_ptr(), Raylib_ImageDrawImageRec);
    register_callback(collector_context, c"raylib::ImageDrawImageRec".as_ptr(), Raylib_ImageDrawImageRec);
    register_callback(collector_context, c"raylib::image_draw_image_rec".as_ptr(), Raylib_ImageDrawImageRec);
    register_callback(collector_context, c"ImageDrawImageRec".as_ptr(), Raylib_ImageDrawImageRec);
    register_callback(collector_context, c"image_draw_image_rec".as_ptr(), Raylib_ImageDrawImageRec);
    register_callback(collector_context, c"Raylib::ImageDrawImagePro".as_ptr(), Raylib_ImageDrawImagePro);
    register_callback(collector_context, c"Raylib::image_draw_image_pro".as_ptr(), Raylib_ImageDrawImagePro);
    register_callback(collector_context, c"raylib::ImageDrawImagePro".as_ptr(), Raylib_ImageDrawImagePro);
    register_callback(collector_context, c"raylib::image_draw_image_pro".as_ptr(), Raylib_ImageDrawImagePro);
    register_callback(collector_context, c"ImageDrawImagePro".as_ptr(), Raylib_ImageDrawImagePro);
    register_callback(collector_context, c"image_draw_image_pro".as_ptr(), Raylib_ImageDrawImagePro);
    register_callback(collector_context, c"Raylib::ImageDrawText".as_ptr(), Raylib_ImageDrawText);
    register_callback(collector_context, c"Raylib::image_draw_text".as_ptr(), Raylib_ImageDrawText);
    register_callback(collector_context, c"raylib::ImageDrawText".as_ptr(), Raylib_ImageDrawText);
    register_callback(collector_context, c"raylib::image_draw_text".as_ptr(), Raylib_ImageDrawText);
    register_callback(collector_context, c"ImageDrawText".as_ptr(), Raylib_ImageDrawText);
    register_callback(collector_context, c"image_draw_text".as_ptr(), Raylib_ImageDrawText);
    register_callback(collector_context, c"Raylib::ImageDrawTextEx".as_ptr(), Raylib_ImageDrawTextEx);
    register_callback(collector_context, c"Raylib::image_draw_text_ex".as_ptr(), Raylib_ImageDrawTextEx);
    register_callback(collector_context, c"raylib::ImageDrawTextEx".as_ptr(), Raylib_ImageDrawTextEx);
    register_callback(collector_context, c"raylib::image_draw_text_ex".as_ptr(), Raylib_ImageDrawTextEx);
    register_callback(collector_context, c"ImageDrawTextEx".as_ptr(), Raylib_ImageDrawTextEx);
    register_callback(collector_context, c"image_draw_text_ex".as_ptr(), Raylib_ImageDrawTextEx);
    register_callback(collector_context, c"Raylib::ImageDrawTextPro".as_ptr(), Raylib_ImageDrawTextPro);
    register_callback(collector_context, c"Raylib::image_draw_text_pro".as_ptr(), Raylib_ImageDrawTextPro);
    register_callback(collector_context, c"raylib::ImageDrawTextPro".as_ptr(), Raylib_ImageDrawTextPro);
    register_callback(collector_context, c"raylib::image_draw_text_pro".as_ptr(), Raylib_ImageDrawTextPro);
    register_callback(collector_context, c"ImageDrawTextPro".as_ptr(), Raylib_ImageDrawTextPro);
    register_callback(collector_context, c"image_draw_text_pro".as_ptr(), Raylib_ImageDrawTextPro);
    register_callback(collector_context, c"Raylib::LoadTexture".as_ptr(), Raylib_LoadTexture);
    register_callback(collector_context, c"Raylib::load_texture".as_ptr(), Raylib_LoadTexture);
    register_callback(collector_context, c"raylib::LoadTexture".as_ptr(), Raylib_LoadTexture);
    register_callback(collector_context, c"raylib::load_texture".as_ptr(), Raylib_LoadTexture);
    register_callback(collector_context, c"LoadTexture".as_ptr(), Raylib_LoadTexture);
    register_callback(collector_context, c"load_texture".as_ptr(), Raylib_LoadTexture);
    register_callback(collector_context, c"Raylib::LoadTextureFromImage".as_ptr(), Raylib_LoadTextureFromImage);
    register_callback(collector_context, c"Raylib::load_texture_from_image".as_ptr(), Raylib_LoadTextureFromImage);
    register_callback(collector_context, c"raylib::LoadTextureFromImage".as_ptr(), Raylib_LoadTextureFromImage);
    register_callback(collector_context, c"raylib::load_texture_from_image".as_ptr(), Raylib_LoadTextureFromImage);
    register_callback(collector_context, c"LoadTextureFromImage".as_ptr(), Raylib_LoadTextureFromImage);
    register_callback(collector_context, c"load_texture_from_image".as_ptr(), Raylib_LoadTextureFromImage);
    register_callback(collector_context, c"Raylib::LoadTextureCubemap".as_ptr(), Raylib_LoadTextureCubemap);
    register_callback(collector_context, c"Raylib::load_texture_cubemap".as_ptr(), Raylib_LoadTextureCubemap);
    register_callback(collector_context, c"raylib::LoadTextureCubemap".as_ptr(), Raylib_LoadTextureCubemap);
    register_callback(collector_context, c"raylib::load_texture_cubemap".as_ptr(), Raylib_LoadTextureCubemap);
    register_callback(collector_context, c"LoadTextureCubemap".as_ptr(), Raylib_LoadTextureCubemap);
    register_callback(collector_context, c"load_texture_cubemap".as_ptr(), Raylib_LoadTextureCubemap);
    register_callback(collector_context, c"Raylib::LoadRenderTexture".as_ptr(), Raylib_LoadRenderTexture);
    register_callback(collector_context, c"Raylib::load_render_texture".as_ptr(), Raylib_LoadRenderTexture);
    register_callback(collector_context, c"raylib::LoadRenderTexture".as_ptr(), Raylib_LoadRenderTexture);
    register_callback(collector_context, c"raylib::load_render_texture".as_ptr(), Raylib_LoadRenderTexture);
    register_callback(collector_context, c"LoadRenderTexture".as_ptr(), Raylib_LoadRenderTexture);
    register_callback(collector_context, c"load_render_texture".as_ptr(), Raylib_LoadRenderTexture);
    register_callback(collector_context, c"Raylib::LoadRenderTextureEx".as_ptr(), Raylib_LoadRenderTextureEx);
    register_callback(collector_context, c"Raylib::load_render_texture_ex".as_ptr(), Raylib_LoadRenderTextureEx);
    register_callback(collector_context, c"raylib::LoadRenderTextureEx".as_ptr(), Raylib_LoadRenderTextureEx);
    register_callback(collector_context, c"raylib::load_render_texture_ex".as_ptr(), Raylib_LoadRenderTextureEx);
    register_callback(collector_context, c"LoadRenderTextureEx".as_ptr(), Raylib_LoadRenderTextureEx);
    register_callback(collector_context, c"load_render_texture_ex".as_ptr(), Raylib_LoadRenderTextureEx);
    register_callback(collector_context, c"Raylib::IsTextureValid".as_ptr(), Raylib_IsTextureValid);
    register_callback(collector_context, c"Raylib::is_texture_valid".as_ptr(), Raylib_IsTextureValid);
    register_callback(collector_context, c"raylib::IsTextureValid".as_ptr(), Raylib_IsTextureValid);
    register_callback(collector_context, c"raylib::is_texture_valid".as_ptr(), Raylib_IsTextureValid);
    register_callback(collector_context, c"IsTextureValid".as_ptr(), Raylib_IsTextureValid);
    register_callback(collector_context, c"is_texture_valid".as_ptr(), Raylib_IsTextureValid);
    register_callback(collector_context, c"Raylib::UnloadTexture".as_ptr(), Raylib_UnloadTexture);
    register_callback(collector_context, c"Raylib::unload_texture".as_ptr(), Raylib_UnloadTexture);
    register_callback(collector_context, c"raylib::UnloadTexture".as_ptr(), Raylib_UnloadTexture);
    register_callback(collector_context, c"raylib::unload_texture".as_ptr(), Raylib_UnloadTexture);
    register_callback(collector_context, c"UnloadTexture".as_ptr(), Raylib_UnloadTexture);
    register_callback(collector_context, c"unload_texture".as_ptr(), Raylib_UnloadTexture);
    register_callback(collector_context, c"Raylib::IsRenderTextureValid".as_ptr(), Raylib_IsRenderTextureValid);
    register_callback(collector_context, c"Raylib::is_render_texture_valid".as_ptr(), Raylib_IsRenderTextureValid);
    register_callback(collector_context, c"raylib::IsRenderTextureValid".as_ptr(), Raylib_IsRenderTextureValid);
    register_callback(collector_context, c"raylib::is_render_texture_valid".as_ptr(), Raylib_IsRenderTextureValid);
    register_callback(collector_context, c"IsRenderTextureValid".as_ptr(), Raylib_IsRenderTextureValid);
    register_callback(collector_context, c"is_render_texture_valid".as_ptr(), Raylib_IsRenderTextureValid);
    register_callback(collector_context, c"Raylib::UnloadRenderTexture".as_ptr(), Raylib_UnloadRenderTexture);
    register_callback(collector_context, c"Raylib::unload_render_texture".as_ptr(), Raylib_UnloadRenderTexture);
    register_callback(collector_context, c"raylib::UnloadRenderTexture".as_ptr(), Raylib_UnloadRenderTexture);
    register_callback(collector_context, c"raylib::unload_render_texture".as_ptr(), Raylib_UnloadRenderTexture);
    register_callback(collector_context, c"UnloadRenderTexture".as_ptr(), Raylib_UnloadRenderTexture);
    register_callback(collector_context, c"unload_render_texture".as_ptr(), Raylib_UnloadRenderTexture);
    register_callback(collector_context, c"Raylib::UpdateTexture".as_ptr(), Raylib_UpdateTexture);
    register_callback(collector_context, c"Raylib::update_texture".as_ptr(), Raylib_UpdateTexture);
    register_callback(collector_context, c"raylib::UpdateTexture".as_ptr(), Raylib_UpdateTexture);
    register_callback(collector_context, c"raylib::update_texture".as_ptr(), Raylib_UpdateTexture);
    register_callback(collector_context, c"UpdateTexture".as_ptr(), Raylib_UpdateTexture);
    register_callback(collector_context, c"update_texture".as_ptr(), Raylib_UpdateTexture);
    register_callback(collector_context, c"Raylib::UpdateTextureRec".as_ptr(), Raylib_UpdateTextureRec);
    register_callback(collector_context, c"Raylib::update_texture_rec".as_ptr(), Raylib_UpdateTextureRec);
    register_callback(collector_context, c"raylib::UpdateTextureRec".as_ptr(), Raylib_UpdateTextureRec);
    register_callback(collector_context, c"raylib::update_texture_rec".as_ptr(), Raylib_UpdateTextureRec);
    register_callback(collector_context, c"UpdateTextureRec".as_ptr(), Raylib_UpdateTextureRec);
    register_callback(collector_context, c"update_texture_rec".as_ptr(), Raylib_UpdateTextureRec);
    register_callback(collector_context, c"Raylib::GenTextureMipmaps".as_ptr(), Raylib_GenTextureMipmaps);
    register_callback(collector_context, c"Raylib::gen_texture_mipmaps".as_ptr(), Raylib_GenTextureMipmaps);
    register_callback(collector_context, c"raylib::GenTextureMipmaps".as_ptr(), Raylib_GenTextureMipmaps);
    register_callback(collector_context, c"raylib::gen_texture_mipmaps".as_ptr(), Raylib_GenTextureMipmaps);
    register_callback(collector_context, c"GenTextureMipmaps".as_ptr(), Raylib_GenTextureMipmaps);
    register_callback(collector_context, c"gen_texture_mipmaps".as_ptr(), Raylib_GenTextureMipmaps);
    register_callback(collector_context, c"Raylib::SetTextureFilter".as_ptr(), Raylib_SetTextureFilter);
    register_callback(collector_context, c"Raylib::set_texture_filter".as_ptr(), Raylib_SetTextureFilter);
    register_callback(collector_context, c"raylib::SetTextureFilter".as_ptr(), Raylib_SetTextureFilter);
    register_callback(collector_context, c"raylib::set_texture_filter".as_ptr(), Raylib_SetTextureFilter);
    register_callback(collector_context, c"SetTextureFilter".as_ptr(), Raylib_SetTextureFilter);
    register_callback(collector_context, c"set_texture_filter".as_ptr(), Raylib_SetTextureFilter);
    register_callback(collector_context, c"Raylib::SetTextureWrap".as_ptr(), Raylib_SetTextureWrap);
    register_callback(collector_context, c"Raylib::set_texture_wrap".as_ptr(), Raylib_SetTextureWrap);
    register_callback(collector_context, c"raylib::SetTextureWrap".as_ptr(), Raylib_SetTextureWrap);
    register_callback(collector_context, c"raylib::set_texture_wrap".as_ptr(), Raylib_SetTextureWrap);
    register_callback(collector_context, c"SetTextureWrap".as_ptr(), Raylib_SetTextureWrap);
    register_callback(collector_context, c"set_texture_wrap".as_ptr(), Raylib_SetTextureWrap);
    register_callback(collector_context, c"Raylib::DrawTexture".as_ptr(), Raylib_DrawTexture);
    register_callback(collector_context, c"Raylib::draw_texture".as_ptr(), Raylib_DrawTexture);
    register_callback(collector_context, c"raylib::DrawTexture".as_ptr(), Raylib_DrawTexture);
    register_callback(collector_context, c"raylib::draw_texture".as_ptr(), Raylib_DrawTexture);
    register_callback(collector_context, c"DrawTexture".as_ptr(), Raylib_DrawTexture);
    register_callback(collector_context, c"draw_texture".as_ptr(), Raylib_DrawTexture);
    register_callback(collector_context, c"Raylib::DrawTextureV".as_ptr(), Raylib_DrawTextureV);
    register_callback(collector_context, c"Raylib::draw_texture_v".as_ptr(), Raylib_DrawTextureV);
    register_callback(collector_context, c"raylib::DrawTextureV".as_ptr(), Raylib_DrawTextureV);
    register_callback(collector_context, c"raylib::draw_texture_v".as_ptr(), Raylib_DrawTextureV);
    register_callback(collector_context, c"DrawTextureV".as_ptr(), Raylib_DrawTextureV);
    register_callback(collector_context, c"draw_texture_v".as_ptr(), Raylib_DrawTextureV);
    register_callback(collector_context, c"Raylib::DrawTextureEx".as_ptr(), Raylib_DrawTextureEx);
    register_callback(collector_context, c"Raylib::draw_texture_ex".as_ptr(), Raylib_DrawTextureEx);
    register_callback(collector_context, c"raylib::DrawTextureEx".as_ptr(), Raylib_DrawTextureEx);
    register_callback(collector_context, c"raylib::draw_texture_ex".as_ptr(), Raylib_DrawTextureEx);
    register_callback(collector_context, c"DrawTextureEx".as_ptr(), Raylib_DrawTextureEx);
    register_callback(collector_context, c"draw_texture_ex".as_ptr(), Raylib_DrawTextureEx);
    register_callback(collector_context, c"Raylib::DrawTextureRec".as_ptr(), Raylib_DrawTextureRec);
    register_callback(collector_context, c"Raylib::draw_texture_rec".as_ptr(), Raylib_DrawTextureRec);
    register_callback(collector_context, c"raylib::DrawTextureRec".as_ptr(), Raylib_DrawTextureRec);
    register_callback(collector_context, c"raylib::draw_texture_rec".as_ptr(), Raylib_DrawTextureRec);
    register_callback(collector_context, c"DrawTextureRec".as_ptr(), Raylib_DrawTextureRec);
    register_callback(collector_context, c"draw_texture_rec".as_ptr(), Raylib_DrawTextureRec);
    register_callback(collector_context, c"Raylib::DrawTexturePro".as_ptr(), Raylib_DrawTexturePro);
    register_callback(collector_context, c"Raylib::draw_texture_pro".as_ptr(), Raylib_DrawTexturePro);
    register_callback(collector_context, c"raylib::DrawTexturePro".as_ptr(), Raylib_DrawTexturePro);
    register_callback(collector_context, c"raylib::draw_texture_pro".as_ptr(), Raylib_DrawTexturePro);
    register_callback(collector_context, c"DrawTexturePro".as_ptr(), Raylib_DrawTexturePro);
    register_callback(collector_context, c"draw_texture_pro".as_ptr(), Raylib_DrawTexturePro);
    register_callback(collector_context, c"Raylib::DrawTextureNPatch".as_ptr(), Raylib_DrawTextureNPatch);
    register_callback(collector_context, c"Raylib::draw_texture_n_patch".as_ptr(), Raylib_DrawTextureNPatch);
    register_callback(collector_context, c"raylib::DrawTextureNPatch".as_ptr(), Raylib_DrawTextureNPatch);
    register_callback(collector_context, c"raylib::draw_texture_n_patch".as_ptr(), Raylib_DrawTextureNPatch);
    register_callback(collector_context, c"DrawTextureNPatch".as_ptr(), Raylib_DrawTextureNPatch);
    register_callback(collector_context, c"draw_texture_n_patch".as_ptr(), Raylib_DrawTextureNPatch);
    register_callback(collector_context, c"Raylib::ColorIsEqual".as_ptr(), Raylib_ColorIsEqual);
    register_callback(collector_context, c"Raylib::color_is_equal".as_ptr(), Raylib_ColorIsEqual);
    register_callback(collector_context, c"raylib::ColorIsEqual".as_ptr(), Raylib_ColorIsEqual);
    register_callback(collector_context, c"raylib::color_is_equal".as_ptr(), Raylib_ColorIsEqual);
    register_callback(collector_context, c"ColorIsEqual".as_ptr(), Raylib_ColorIsEqual);
    register_callback(collector_context, c"color_is_equal".as_ptr(), Raylib_ColorIsEqual);
    register_callback(collector_context, c"Raylib::Fade".as_ptr(), Raylib_Fade);
    register_callback(collector_context, c"Raylib::fade".as_ptr(), Raylib_Fade);
    register_callback(collector_context, c"raylib::Fade".as_ptr(), Raylib_Fade);
    register_callback(collector_context, c"raylib::fade".as_ptr(), Raylib_Fade);
    register_callback(collector_context, c"Fade".as_ptr(), Raylib_Fade);
    register_callback(collector_context, c"fade".as_ptr(), Raylib_Fade);
    register_callback(collector_context, c"Raylib::ColorToInt".as_ptr(), Raylib_ColorToInt);
    register_callback(collector_context, c"Raylib::color_to_int".as_ptr(), Raylib_ColorToInt);
    register_callback(collector_context, c"raylib::ColorToInt".as_ptr(), Raylib_ColorToInt);
    register_callback(collector_context, c"raylib::color_to_int".as_ptr(), Raylib_ColorToInt);
    register_callback(collector_context, c"ColorToInt".as_ptr(), Raylib_ColorToInt);
    register_callback(collector_context, c"color_to_int".as_ptr(), Raylib_ColorToInt);
    register_callback(collector_context, c"Raylib::ColorNormalize".as_ptr(), Raylib_ColorNormalize);
    register_callback(collector_context, c"Raylib::color_normalize".as_ptr(), Raylib_ColorNormalize);
    register_callback(collector_context, c"raylib::ColorNormalize".as_ptr(), Raylib_ColorNormalize);
    register_callback(collector_context, c"raylib::color_normalize".as_ptr(), Raylib_ColorNormalize);
    register_callback(collector_context, c"ColorNormalize".as_ptr(), Raylib_ColorNormalize);
    register_callback(collector_context, c"color_normalize".as_ptr(), Raylib_ColorNormalize);
    register_callback(collector_context, c"Raylib::ColorFromNormalized".as_ptr(), Raylib_ColorFromNormalized);
    register_callback(collector_context, c"Raylib::color_from_normalized".as_ptr(), Raylib_ColorFromNormalized);
    register_callback(collector_context, c"raylib::ColorFromNormalized".as_ptr(), Raylib_ColorFromNormalized);
    register_callback(collector_context, c"raylib::color_from_normalized".as_ptr(), Raylib_ColorFromNormalized);
    register_callback(collector_context, c"ColorFromNormalized".as_ptr(), Raylib_ColorFromNormalized);
    register_callback(collector_context, c"color_from_normalized".as_ptr(), Raylib_ColorFromNormalized);
    register_callback(collector_context, c"Raylib::ColorToHSV".as_ptr(), Raylib_ColorToHSV);
    register_callback(collector_context, c"Raylib::color_to_hsv".as_ptr(), Raylib_ColorToHSV);
    register_callback(collector_context, c"raylib::ColorToHSV".as_ptr(), Raylib_ColorToHSV);
    register_callback(collector_context, c"raylib::color_to_hsv".as_ptr(), Raylib_ColorToHSV);
    register_callback(collector_context, c"ColorToHSV".as_ptr(), Raylib_ColorToHSV);
    register_callback(collector_context, c"color_to_hsv".as_ptr(), Raylib_ColorToHSV);
    register_callback(collector_context, c"Raylib::ColorFromHSV".as_ptr(), Raylib_ColorFromHSV);
    register_callback(collector_context, c"Raylib::color_from_hsv".as_ptr(), Raylib_ColorFromHSV);
    register_callback(collector_context, c"raylib::ColorFromHSV".as_ptr(), Raylib_ColorFromHSV);
    register_callback(collector_context, c"raylib::color_from_hsv".as_ptr(), Raylib_ColorFromHSV);
    register_callback(collector_context, c"ColorFromHSV".as_ptr(), Raylib_ColorFromHSV);
    register_callback(collector_context, c"color_from_hsv".as_ptr(), Raylib_ColorFromHSV);
    register_callback(collector_context, c"Raylib::ColorTint".as_ptr(), Raylib_ColorTint);
    register_callback(collector_context, c"Raylib::color_tint".as_ptr(), Raylib_ColorTint);
    register_callback(collector_context, c"raylib::ColorTint".as_ptr(), Raylib_ColorTint);
    register_callback(collector_context, c"raylib::color_tint".as_ptr(), Raylib_ColorTint);
    register_callback(collector_context, c"ColorTint".as_ptr(), Raylib_ColorTint);
    register_callback(collector_context, c"color_tint".as_ptr(), Raylib_ColorTint);
    register_callback(collector_context, c"Raylib::ColorBrightness".as_ptr(), Raylib_ColorBrightness);
    register_callback(collector_context, c"Raylib::color_brightness".as_ptr(), Raylib_ColorBrightness);
    register_callback(collector_context, c"raylib::ColorBrightness".as_ptr(), Raylib_ColorBrightness);
    register_callback(collector_context, c"raylib::color_brightness".as_ptr(), Raylib_ColorBrightness);
    register_callback(collector_context, c"ColorBrightness".as_ptr(), Raylib_ColorBrightness);
    register_callback(collector_context, c"color_brightness".as_ptr(), Raylib_ColorBrightness);
    register_callback(collector_context, c"Raylib::ColorContrast".as_ptr(), Raylib_ColorContrast);
    register_callback(collector_context, c"Raylib::color_contrast".as_ptr(), Raylib_ColorContrast);
    register_callback(collector_context, c"raylib::ColorContrast".as_ptr(), Raylib_ColorContrast);
    register_callback(collector_context, c"raylib::color_contrast".as_ptr(), Raylib_ColorContrast);
    register_callback(collector_context, c"ColorContrast".as_ptr(), Raylib_ColorContrast);
    register_callback(collector_context, c"color_contrast".as_ptr(), Raylib_ColorContrast);
    register_callback(collector_context, c"Raylib::ColorAlpha".as_ptr(), Raylib_ColorAlpha);
    register_callback(collector_context, c"Raylib::color_alpha".as_ptr(), Raylib_ColorAlpha);
    register_callback(collector_context, c"raylib::ColorAlpha".as_ptr(), Raylib_ColorAlpha);
    register_callback(collector_context, c"raylib::color_alpha".as_ptr(), Raylib_ColorAlpha);
    register_callback(collector_context, c"ColorAlpha".as_ptr(), Raylib_ColorAlpha);
    register_callback(collector_context, c"color_alpha".as_ptr(), Raylib_ColorAlpha);
    register_callback(collector_context, c"Raylib::ColorAlphaBlend".as_ptr(), Raylib_ColorAlphaBlend);
    register_callback(collector_context, c"Raylib::color_alpha_blend".as_ptr(), Raylib_ColorAlphaBlend);
    register_callback(collector_context, c"raylib::ColorAlphaBlend".as_ptr(), Raylib_ColorAlphaBlend);
    register_callback(collector_context, c"raylib::color_alpha_blend".as_ptr(), Raylib_ColorAlphaBlend);
    register_callback(collector_context, c"ColorAlphaBlend".as_ptr(), Raylib_ColorAlphaBlend);
    register_callback(collector_context, c"color_alpha_blend".as_ptr(), Raylib_ColorAlphaBlend);
    register_callback(collector_context, c"Raylib::ColorLerp".as_ptr(), Raylib_ColorLerp);
    register_callback(collector_context, c"Raylib::color_lerp".as_ptr(), Raylib_ColorLerp);
    register_callback(collector_context, c"raylib::ColorLerp".as_ptr(), Raylib_ColorLerp);
    register_callback(collector_context, c"raylib::color_lerp".as_ptr(), Raylib_ColorLerp);
    register_callback(collector_context, c"ColorLerp".as_ptr(), Raylib_ColorLerp);
    register_callback(collector_context, c"color_lerp".as_ptr(), Raylib_ColorLerp);
    register_callback(collector_context, c"Raylib::GetColor".as_ptr(), Raylib_GetColor);
    register_callback(collector_context, c"Raylib::get_color".as_ptr(), Raylib_GetColor);
    register_callback(collector_context, c"raylib::GetColor".as_ptr(), Raylib_GetColor);
    register_callback(collector_context, c"raylib::get_color".as_ptr(), Raylib_GetColor);
    register_callback(collector_context, c"GetColor".as_ptr(), Raylib_GetColor);
    register_callback(collector_context, c"get_color".as_ptr(), Raylib_GetColor);
    register_callback(collector_context, c"Raylib::GetPixelColor".as_ptr(), Raylib_GetPixelColor);
    register_callback(collector_context, c"Raylib::get_pixel_color".as_ptr(), Raylib_GetPixelColor);
    register_callback(collector_context, c"raylib::GetPixelColor".as_ptr(), Raylib_GetPixelColor);
    register_callback(collector_context, c"raylib::get_pixel_color".as_ptr(), Raylib_GetPixelColor);
    register_callback(collector_context, c"GetPixelColor".as_ptr(), Raylib_GetPixelColor);
    register_callback(collector_context, c"get_pixel_color".as_ptr(), Raylib_GetPixelColor);
    register_callback(collector_context, c"Raylib::SetPixelColor".as_ptr(), Raylib_SetPixelColor);
    register_callback(collector_context, c"Raylib::set_pixel_color".as_ptr(), Raylib_SetPixelColor);
    register_callback(collector_context, c"raylib::SetPixelColor".as_ptr(), Raylib_SetPixelColor);
    register_callback(collector_context, c"raylib::set_pixel_color".as_ptr(), Raylib_SetPixelColor);
    register_callback(collector_context, c"SetPixelColor".as_ptr(), Raylib_SetPixelColor);
    register_callback(collector_context, c"set_pixel_color".as_ptr(), Raylib_SetPixelColor);
    register_callback(collector_context, c"Raylib::GetPixelDataSize".as_ptr(), Raylib_GetPixelDataSize);
    register_callback(collector_context, c"Raylib::get_pixel_data_size".as_ptr(), Raylib_GetPixelDataSize);
    register_callback(collector_context, c"raylib::GetPixelDataSize".as_ptr(), Raylib_GetPixelDataSize);
    register_callback(collector_context, c"raylib::get_pixel_data_size".as_ptr(), Raylib_GetPixelDataSize);
    register_callback(collector_context, c"GetPixelDataSize".as_ptr(), Raylib_GetPixelDataSize);
    register_callback(collector_context, c"get_pixel_data_size".as_ptr(), Raylib_GetPixelDataSize);
    register_callback(collector_context, c"Raylib::GetFontDefault".as_ptr(), Raylib_GetFontDefault);
    register_callback(collector_context, c"Raylib::get_font_default".as_ptr(), Raylib_GetFontDefault);
    register_callback(collector_context, c"raylib::GetFontDefault".as_ptr(), Raylib_GetFontDefault);
    register_callback(collector_context, c"raylib::get_font_default".as_ptr(), Raylib_GetFontDefault);
    register_callback(collector_context, c"GetFontDefault".as_ptr(), Raylib_GetFontDefault);
    register_callback(collector_context, c"get_font_default".as_ptr(), Raylib_GetFontDefault);
    register_callback(collector_context, c"Raylib::LoadFont".as_ptr(), Raylib_LoadFont);
    register_callback(collector_context, c"Raylib::load_font".as_ptr(), Raylib_LoadFont);
    register_callback(collector_context, c"raylib::LoadFont".as_ptr(), Raylib_LoadFont);
    register_callback(collector_context, c"raylib::load_font".as_ptr(), Raylib_LoadFont);
    register_callback(collector_context, c"LoadFont".as_ptr(), Raylib_LoadFont);
    register_callback(collector_context, c"load_font".as_ptr(), Raylib_LoadFont);
    register_callback(collector_context, c"Raylib::LoadFontEx".as_ptr(), Raylib_LoadFontEx);
    register_callback(collector_context, c"Raylib::load_font_ex".as_ptr(), Raylib_LoadFontEx);
    register_callback(collector_context, c"raylib::LoadFontEx".as_ptr(), Raylib_LoadFontEx);
    register_callback(collector_context, c"raylib::load_font_ex".as_ptr(), Raylib_LoadFontEx);
    register_callback(collector_context, c"LoadFontEx".as_ptr(), Raylib_LoadFontEx);
    register_callback(collector_context, c"load_font_ex".as_ptr(), Raylib_LoadFontEx);
    register_callback(collector_context, c"Raylib::LoadFontFromImage".as_ptr(), Raylib_LoadFontFromImage);
    register_callback(collector_context, c"Raylib::load_font_from_image".as_ptr(), Raylib_LoadFontFromImage);
    register_callback(collector_context, c"raylib::LoadFontFromImage".as_ptr(), Raylib_LoadFontFromImage);
    register_callback(collector_context, c"raylib::load_font_from_image".as_ptr(), Raylib_LoadFontFromImage);
    register_callback(collector_context, c"LoadFontFromImage".as_ptr(), Raylib_LoadFontFromImage);
    register_callback(collector_context, c"load_font_from_image".as_ptr(), Raylib_LoadFontFromImage);
    register_callback(collector_context, c"Raylib::LoadFontFromMemory".as_ptr(), Raylib_LoadFontFromMemory);
    register_callback(collector_context, c"Raylib::load_font_from_memory".as_ptr(), Raylib_LoadFontFromMemory);
    register_callback(collector_context, c"raylib::LoadFontFromMemory".as_ptr(), Raylib_LoadFontFromMemory);
    register_callback(collector_context, c"raylib::load_font_from_memory".as_ptr(), Raylib_LoadFontFromMemory);
    register_callback(collector_context, c"LoadFontFromMemory".as_ptr(), Raylib_LoadFontFromMemory);
    register_callback(collector_context, c"load_font_from_memory".as_ptr(), Raylib_LoadFontFromMemory);
    register_callback(collector_context, c"Raylib::IsFontValid".as_ptr(), Raylib_IsFontValid);
    register_callback(collector_context, c"Raylib::is_font_valid".as_ptr(), Raylib_IsFontValid);
    register_callback(collector_context, c"raylib::IsFontValid".as_ptr(), Raylib_IsFontValid);
    register_callback(collector_context, c"raylib::is_font_valid".as_ptr(), Raylib_IsFontValid);
    register_callback(collector_context, c"IsFontValid".as_ptr(), Raylib_IsFontValid);
    register_callback(collector_context, c"is_font_valid".as_ptr(), Raylib_IsFontValid);
    register_callback(collector_context, c"Raylib::LoadFontData".as_ptr(), Raylib_LoadFontData);
    register_callback(collector_context, c"Raylib::load_font_data".as_ptr(), Raylib_LoadFontData);
    register_callback(collector_context, c"raylib::LoadFontData".as_ptr(), Raylib_LoadFontData);
    register_callback(collector_context, c"raylib::load_font_data".as_ptr(), Raylib_LoadFontData);
    register_callback(collector_context, c"LoadFontData".as_ptr(), Raylib_LoadFontData);
    register_callback(collector_context, c"load_font_data".as_ptr(), Raylib_LoadFontData);
    register_callback(collector_context, c"Raylib::GenImageFontAtlas".as_ptr(), Raylib_GenImageFontAtlas);
    register_callback(collector_context, c"Raylib::gen_image_font_atlas".as_ptr(), Raylib_GenImageFontAtlas);
    register_callback(collector_context, c"raylib::GenImageFontAtlas".as_ptr(), Raylib_GenImageFontAtlas);
    register_callback(collector_context, c"raylib::gen_image_font_atlas".as_ptr(), Raylib_GenImageFontAtlas);
    register_callback(collector_context, c"GenImageFontAtlas".as_ptr(), Raylib_GenImageFontAtlas);
    register_callback(collector_context, c"gen_image_font_atlas".as_ptr(), Raylib_GenImageFontAtlas);
    register_callback(collector_context, c"Raylib::UnloadFontData".as_ptr(), Raylib_UnloadFontData);
    register_callback(collector_context, c"Raylib::unload_font_data".as_ptr(), Raylib_UnloadFontData);
    register_callback(collector_context, c"raylib::UnloadFontData".as_ptr(), Raylib_UnloadFontData);
    register_callback(collector_context, c"raylib::unload_font_data".as_ptr(), Raylib_UnloadFontData);
    register_callback(collector_context, c"UnloadFontData".as_ptr(), Raylib_UnloadFontData);
    register_callback(collector_context, c"unload_font_data".as_ptr(), Raylib_UnloadFontData);
    register_callback(collector_context, c"Raylib::UnloadFont".as_ptr(), Raylib_UnloadFont);
    register_callback(collector_context, c"Raylib::unload_font".as_ptr(), Raylib_UnloadFont);
    register_callback(collector_context, c"raylib::UnloadFont".as_ptr(), Raylib_UnloadFont);
    register_callback(collector_context, c"raylib::unload_font".as_ptr(), Raylib_UnloadFont);
    register_callback(collector_context, c"UnloadFont".as_ptr(), Raylib_UnloadFont);
    register_callback(collector_context, c"unload_font".as_ptr(), Raylib_UnloadFont);
    register_callback(collector_context, c"Raylib::ExportFontAsCode".as_ptr(), Raylib_ExportFontAsCode);
    register_callback(collector_context, c"Raylib::export_font_as_code".as_ptr(), Raylib_ExportFontAsCode);
    register_callback(collector_context, c"raylib::ExportFontAsCode".as_ptr(), Raylib_ExportFontAsCode);
    register_callback(collector_context, c"raylib::export_font_as_code".as_ptr(), Raylib_ExportFontAsCode);
    register_callback(collector_context, c"ExportFontAsCode".as_ptr(), Raylib_ExportFontAsCode);
    register_callback(collector_context, c"export_font_as_code".as_ptr(), Raylib_ExportFontAsCode);
    register_callback(collector_context, c"Raylib::DrawFPS".as_ptr(), Raylib_DrawFPS);
    register_callback(collector_context, c"Raylib::draw_fps".as_ptr(), Raylib_DrawFPS);
    register_callback(collector_context, c"raylib::DrawFPS".as_ptr(), Raylib_DrawFPS);
    register_callback(collector_context, c"raylib::draw_fps".as_ptr(), Raylib_DrawFPS);
    register_callback(collector_context, c"DrawFPS".as_ptr(), Raylib_DrawFPS);
    register_callback(collector_context, c"draw_fps".as_ptr(), Raylib_DrawFPS);
    register_callback(collector_context, c"Raylib::DrawText".as_ptr(), Raylib_DrawText);
    register_callback(collector_context, c"Raylib::draw_text".as_ptr(), Raylib_DrawText);
    register_callback(collector_context, c"raylib::DrawText".as_ptr(), Raylib_DrawText);
    register_callback(collector_context, c"raylib::draw_text".as_ptr(), Raylib_DrawText);
    register_callback(collector_context, c"DrawText".as_ptr(), Raylib_DrawText);
    register_callback(collector_context, c"draw_text".as_ptr(), Raylib_DrawText);
    register_callback(collector_context, c"Raylib::DrawTextEx".as_ptr(), Raylib_DrawTextEx);
    register_callback(collector_context, c"Raylib::draw_text_ex".as_ptr(), Raylib_DrawTextEx);
    register_callback(collector_context, c"raylib::DrawTextEx".as_ptr(), Raylib_DrawTextEx);
    register_callback(collector_context, c"raylib::draw_text_ex".as_ptr(), Raylib_DrawTextEx);
    register_callback(collector_context, c"DrawTextEx".as_ptr(), Raylib_DrawTextEx);
    register_callback(collector_context, c"draw_text_ex".as_ptr(), Raylib_DrawTextEx);
    register_callback(collector_context, c"Raylib::DrawTextPro".as_ptr(), Raylib_DrawTextPro);
    register_callback(collector_context, c"Raylib::draw_text_pro".as_ptr(), Raylib_DrawTextPro);
    register_callback(collector_context, c"raylib::DrawTextPro".as_ptr(), Raylib_DrawTextPro);
    register_callback(collector_context, c"raylib::draw_text_pro".as_ptr(), Raylib_DrawTextPro);
    register_callback(collector_context, c"DrawTextPro".as_ptr(), Raylib_DrawTextPro);
    register_callback(collector_context, c"draw_text_pro".as_ptr(), Raylib_DrawTextPro);
    register_callback(collector_context, c"Raylib::DrawTextCodepoint".as_ptr(), Raylib_DrawTextCodepoint);
    register_callback(collector_context, c"Raylib::draw_text_codepoint".as_ptr(), Raylib_DrawTextCodepoint);
    register_callback(collector_context, c"raylib::DrawTextCodepoint".as_ptr(), Raylib_DrawTextCodepoint);
    register_callback(collector_context, c"raylib::draw_text_codepoint".as_ptr(), Raylib_DrawTextCodepoint);
    register_callback(collector_context, c"DrawTextCodepoint".as_ptr(), Raylib_DrawTextCodepoint);
    register_callback(collector_context, c"draw_text_codepoint".as_ptr(), Raylib_DrawTextCodepoint);
    register_callback(collector_context, c"Raylib::DrawTextCodepoints".as_ptr(), Raylib_DrawTextCodepoints);
    register_callback(collector_context, c"Raylib::draw_text_codepoints".as_ptr(), Raylib_DrawTextCodepoints);
    register_callback(collector_context, c"raylib::DrawTextCodepoints".as_ptr(), Raylib_DrawTextCodepoints);
    register_callback(collector_context, c"raylib::draw_text_codepoints".as_ptr(), Raylib_DrawTextCodepoints);
    register_callback(collector_context, c"DrawTextCodepoints".as_ptr(), Raylib_DrawTextCodepoints);
    register_callback(collector_context, c"draw_text_codepoints".as_ptr(), Raylib_DrawTextCodepoints);
    register_callback(collector_context, c"Raylib::SetTextLineSpacing".as_ptr(), Raylib_SetTextLineSpacing);
    register_callback(collector_context, c"Raylib::set_text_line_spacing".as_ptr(), Raylib_SetTextLineSpacing);
    register_callback(collector_context, c"raylib::SetTextLineSpacing".as_ptr(), Raylib_SetTextLineSpacing);
    register_callback(collector_context, c"raylib::set_text_line_spacing".as_ptr(), Raylib_SetTextLineSpacing);
    register_callback(collector_context, c"SetTextLineSpacing".as_ptr(), Raylib_SetTextLineSpacing);
    register_callback(collector_context, c"set_text_line_spacing".as_ptr(), Raylib_SetTextLineSpacing);
    register_callback(collector_context, c"Raylib::MeasureText".as_ptr(), Raylib_MeasureText);
    register_callback(collector_context, c"Raylib::measure_text".as_ptr(), Raylib_MeasureText);
    register_callback(collector_context, c"raylib::MeasureText".as_ptr(), Raylib_MeasureText);
    register_callback(collector_context, c"raylib::measure_text".as_ptr(), Raylib_MeasureText);
    register_callback(collector_context, c"MeasureText".as_ptr(), Raylib_MeasureText);
    register_callback(collector_context, c"measure_text".as_ptr(), Raylib_MeasureText);
    register_callback(collector_context, c"Raylib::MeasureTextEx".as_ptr(), Raylib_MeasureTextEx);
    register_callback(collector_context, c"Raylib::measure_text_ex".as_ptr(), Raylib_MeasureTextEx);
    register_callback(collector_context, c"raylib::MeasureTextEx".as_ptr(), Raylib_MeasureTextEx);
    register_callback(collector_context, c"raylib::measure_text_ex".as_ptr(), Raylib_MeasureTextEx);
    register_callback(collector_context, c"MeasureTextEx".as_ptr(), Raylib_MeasureTextEx);
    register_callback(collector_context, c"measure_text_ex".as_ptr(), Raylib_MeasureTextEx);
    register_callback(collector_context, c"Raylib::MeasureTextCodepoints".as_ptr(), Raylib_MeasureTextCodepoints);
    register_callback(collector_context, c"Raylib::measure_text_codepoints".as_ptr(), Raylib_MeasureTextCodepoints);
    register_callback(collector_context, c"raylib::MeasureTextCodepoints".as_ptr(), Raylib_MeasureTextCodepoints);
    register_callback(collector_context, c"raylib::measure_text_codepoints".as_ptr(), Raylib_MeasureTextCodepoints);
    register_callback(collector_context, c"MeasureTextCodepoints".as_ptr(), Raylib_MeasureTextCodepoints);
    register_callback(collector_context, c"measure_text_codepoints".as_ptr(), Raylib_MeasureTextCodepoints);
    register_callback(collector_context, c"Raylib::GetGlyphIndex".as_ptr(), Raylib_GetGlyphIndex);
    register_callback(collector_context, c"Raylib::get_glyph_index".as_ptr(), Raylib_GetGlyphIndex);
    register_callback(collector_context, c"raylib::GetGlyphIndex".as_ptr(), Raylib_GetGlyphIndex);
    register_callback(collector_context, c"raylib::get_glyph_index".as_ptr(), Raylib_GetGlyphIndex);
    register_callback(collector_context, c"GetGlyphIndex".as_ptr(), Raylib_GetGlyphIndex);
    register_callback(collector_context, c"get_glyph_index".as_ptr(), Raylib_GetGlyphIndex);
    register_callback(collector_context, c"Raylib::GetGlyphInfo".as_ptr(), Raylib_GetGlyphInfo);
    register_callback(collector_context, c"Raylib::get_glyph_info".as_ptr(), Raylib_GetGlyphInfo);
    register_callback(collector_context, c"raylib::GetGlyphInfo".as_ptr(), Raylib_GetGlyphInfo);
    register_callback(collector_context, c"raylib::get_glyph_info".as_ptr(), Raylib_GetGlyphInfo);
    register_callback(collector_context, c"GetGlyphInfo".as_ptr(), Raylib_GetGlyphInfo);
    register_callback(collector_context, c"get_glyph_info".as_ptr(), Raylib_GetGlyphInfo);
    register_callback(collector_context, c"Raylib::GetGlyphAtlasRec".as_ptr(), Raylib_GetGlyphAtlasRec);
    register_callback(collector_context, c"Raylib::get_glyph_atlas_rec".as_ptr(), Raylib_GetGlyphAtlasRec);
    register_callback(collector_context, c"raylib::GetGlyphAtlasRec".as_ptr(), Raylib_GetGlyphAtlasRec);
    register_callback(collector_context, c"raylib::get_glyph_atlas_rec".as_ptr(), Raylib_GetGlyphAtlasRec);
    register_callback(collector_context, c"GetGlyphAtlasRec".as_ptr(), Raylib_GetGlyphAtlasRec);
    register_callback(collector_context, c"get_glyph_atlas_rec".as_ptr(), Raylib_GetGlyphAtlasRec);
    register_callback(collector_context, c"Raylib::LoadUTF8".as_ptr(), Raylib_LoadUTF8);
    register_callback(collector_context, c"Raylib::load_utf8".as_ptr(), Raylib_LoadUTF8);
    register_callback(collector_context, c"raylib::LoadUTF8".as_ptr(), Raylib_LoadUTF8);
    register_callback(collector_context, c"raylib::load_utf8".as_ptr(), Raylib_LoadUTF8);
    register_callback(collector_context, c"LoadUTF8".as_ptr(), Raylib_LoadUTF8);
    register_callback(collector_context, c"load_utf8".as_ptr(), Raylib_LoadUTF8);
    register_callback(collector_context, c"Raylib::UnloadUTF8".as_ptr(), Raylib_UnloadUTF8);
    register_callback(collector_context, c"Raylib::unload_utf8".as_ptr(), Raylib_UnloadUTF8);
    register_callback(collector_context, c"raylib::UnloadUTF8".as_ptr(), Raylib_UnloadUTF8);
    register_callback(collector_context, c"raylib::unload_utf8".as_ptr(), Raylib_UnloadUTF8);
    register_callback(collector_context, c"UnloadUTF8".as_ptr(), Raylib_UnloadUTF8);
    register_callback(collector_context, c"unload_utf8".as_ptr(), Raylib_UnloadUTF8);
    register_callback(collector_context, c"Raylib::LoadCodepoints".as_ptr(), Raylib_LoadCodepoints);
    register_callback(collector_context, c"Raylib::load_codepoints".as_ptr(), Raylib_LoadCodepoints);
    register_callback(collector_context, c"raylib::LoadCodepoints".as_ptr(), Raylib_LoadCodepoints);
    register_callback(collector_context, c"raylib::load_codepoints".as_ptr(), Raylib_LoadCodepoints);
    register_callback(collector_context, c"LoadCodepoints".as_ptr(), Raylib_LoadCodepoints);
    register_callback(collector_context, c"load_codepoints".as_ptr(), Raylib_LoadCodepoints);
    register_callback(collector_context, c"Raylib::UnloadCodepoints".as_ptr(), Raylib_UnloadCodepoints);
    register_callback(collector_context, c"Raylib::unload_codepoints".as_ptr(), Raylib_UnloadCodepoints);
    register_callback(collector_context, c"raylib::UnloadCodepoints".as_ptr(), Raylib_UnloadCodepoints);
    register_callback(collector_context, c"raylib::unload_codepoints".as_ptr(), Raylib_UnloadCodepoints);
    register_callback(collector_context, c"UnloadCodepoints".as_ptr(), Raylib_UnloadCodepoints);
    register_callback(collector_context, c"unload_codepoints".as_ptr(), Raylib_UnloadCodepoints);
    register_callback(collector_context, c"Raylib::GetCodepointCount".as_ptr(), Raylib_GetCodepointCount);
    register_callback(collector_context, c"Raylib::get_codepoint_count".as_ptr(), Raylib_GetCodepointCount);
    register_callback(collector_context, c"raylib::GetCodepointCount".as_ptr(), Raylib_GetCodepointCount);
    register_callback(collector_context, c"raylib::get_codepoint_count".as_ptr(), Raylib_GetCodepointCount);
    register_callback(collector_context, c"GetCodepointCount".as_ptr(), Raylib_GetCodepointCount);
    register_callback(collector_context, c"get_codepoint_count".as_ptr(), Raylib_GetCodepointCount);
    register_callback(collector_context, c"Raylib::GetCodepoint".as_ptr(), Raylib_GetCodepoint);
    register_callback(collector_context, c"Raylib::get_codepoint".as_ptr(), Raylib_GetCodepoint);
    register_callback(collector_context, c"raylib::GetCodepoint".as_ptr(), Raylib_GetCodepoint);
    register_callback(collector_context, c"raylib::get_codepoint".as_ptr(), Raylib_GetCodepoint);
    register_callback(collector_context, c"GetCodepoint".as_ptr(), Raylib_GetCodepoint);
    register_callback(collector_context, c"get_codepoint".as_ptr(), Raylib_GetCodepoint);
    register_callback(collector_context, c"Raylib::GetCodepointNext".as_ptr(), Raylib_GetCodepointNext);
    register_callback(collector_context, c"Raylib::get_codepoint_next".as_ptr(), Raylib_GetCodepointNext);
    register_callback(collector_context, c"raylib::GetCodepointNext".as_ptr(), Raylib_GetCodepointNext);
    register_callback(collector_context, c"raylib::get_codepoint_next".as_ptr(), Raylib_GetCodepointNext);
    register_callback(collector_context, c"GetCodepointNext".as_ptr(), Raylib_GetCodepointNext);
    register_callback(collector_context, c"get_codepoint_next".as_ptr(), Raylib_GetCodepointNext);
    register_callback(collector_context, c"Raylib::GetCodepointPrevious".as_ptr(), Raylib_GetCodepointPrevious);
    register_callback(collector_context, c"Raylib::get_codepoint_previous".as_ptr(), Raylib_GetCodepointPrevious);
    register_callback(collector_context, c"raylib::GetCodepointPrevious".as_ptr(), Raylib_GetCodepointPrevious);
    register_callback(collector_context, c"raylib::get_codepoint_previous".as_ptr(), Raylib_GetCodepointPrevious);
    register_callback(collector_context, c"GetCodepointPrevious".as_ptr(), Raylib_GetCodepointPrevious);
    register_callback(collector_context, c"get_codepoint_previous".as_ptr(), Raylib_GetCodepointPrevious);
    register_callback(collector_context, c"Raylib::CodepointToUTF8".as_ptr(), Raylib_CodepointToUTF8);
    register_callback(collector_context, c"Raylib::codepoint_to_utf8".as_ptr(), Raylib_CodepointToUTF8);
    register_callback(collector_context, c"raylib::CodepointToUTF8".as_ptr(), Raylib_CodepointToUTF8);
    register_callback(collector_context, c"raylib::codepoint_to_utf8".as_ptr(), Raylib_CodepointToUTF8);
    register_callback(collector_context, c"CodepointToUTF8".as_ptr(), Raylib_CodepointToUTF8);
    register_callback(collector_context, c"codepoint_to_utf8".as_ptr(), Raylib_CodepointToUTF8);
    register_callback(collector_context, c"Raylib::LoadTextLines".as_ptr(), Raylib_LoadTextLines);
    register_callback(collector_context, c"Raylib::load_text_lines".as_ptr(), Raylib_LoadTextLines);
    register_callback(collector_context, c"raylib::LoadTextLines".as_ptr(), Raylib_LoadTextLines);
    register_callback(collector_context, c"raylib::load_text_lines".as_ptr(), Raylib_LoadTextLines);
    register_callback(collector_context, c"LoadTextLines".as_ptr(), Raylib_LoadTextLines);
    register_callback(collector_context, c"load_text_lines".as_ptr(), Raylib_LoadTextLines);
    register_callback(collector_context, c"Raylib::UnloadTextLines".as_ptr(), Raylib_UnloadTextLines);
    register_callback(collector_context, c"Raylib::unload_text_lines".as_ptr(), Raylib_UnloadTextLines);
    register_callback(collector_context, c"raylib::UnloadTextLines".as_ptr(), Raylib_UnloadTextLines);
    register_callback(collector_context, c"raylib::unload_text_lines".as_ptr(), Raylib_UnloadTextLines);
    register_callback(collector_context, c"UnloadTextLines".as_ptr(), Raylib_UnloadTextLines);
    register_callback(collector_context, c"unload_text_lines".as_ptr(), Raylib_UnloadTextLines);
    register_callback(collector_context, c"Raylib::TextCopy".as_ptr(), Raylib_TextCopy);
    register_callback(collector_context, c"Raylib::text_copy".as_ptr(), Raylib_TextCopy);
    register_callback(collector_context, c"raylib::TextCopy".as_ptr(), Raylib_TextCopy);
    register_callback(collector_context, c"raylib::text_copy".as_ptr(), Raylib_TextCopy);
    register_callback(collector_context, c"TextCopy".as_ptr(), Raylib_TextCopy);
    register_callback(collector_context, c"text_copy".as_ptr(), Raylib_TextCopy);
    register_callback(collector_context, c"Raylib::TextIsEqual".as_ptr(), Raylib_TextIsEqual);
    register_callback(collector_context, c"Raylib::text_is_equal".as_ptr(), Raylib_TextIsEqual);
    register_callback(collector_context, c"raylib::TextIsEqual".as_ptr(), Raylib_TextIsEqual);
    register_callback(collector_context, c"raylib::text_is_equal".as_ptr(), Raylib_TextIsEqual);
    register_callback(collector_context, c"TextIsEqual".as_ptr(), Raylib_TextIsEqual);
    register_callback(collector_context, c"text_is_equal".as_ptr(), Raylib_TextIsEqual);
    register_callback(collector_context, c"Raylib::TextLength".as_ptr(), Raylib_TextLength);
    register_callback(collector_context, c"Raylib::text_length".as_ptr(), Raylib_TextLength);
    register_callback(collector_context, c"raylib::TextLength".as_ptr(), Raylib_TextLength);
    register_callback(collector_context, c"raylib::text_length".as_ptr(), Raylib_TextLength);
    register_callback(collector_context, c"TextLength".as_ptr(), Raylib_TextLength);
    register_callback(collector_context, c"text_length".as_ptr(), Raylib_TextLength);
    register_callback(collector_context, c"Raylib::TextFormat".as_ptr(), Raylib_TextFormat);
    register_callback(collector_context, c"Raylib::text_format".as_ptr(), Raylib_TextFormat);
    register_callback(collector_context, c"raylib::TextFormat".as_ptr(), Raylib_TextFormat);
    register_callback(collector_context, c"raylib::text_format".as_ptr(), Raylib_TextFormat);
    register_callback(collector_context, c"TextFormat".as_ptr(), Raylib_TextFormat);
    register_callback(collector_context, c"text_format".as_ptr(), Raylib_TextFormat);
    register_callback(collector_context, c"Raylib::TextSubtext".as_ptr(), Raylib_TextSubtext);
    register_callback(collector_context, c"Raylib::text_subtext".as_ptr(), Raylib_TextSubtext);
    register_callback(collector_context, c"raylib::TextSubtext".as_ptr(), Raylib_TextSubtext);
    register_callback(collector_context, c"raylib::text_subtext".as_ptr(), Raylib_TextSubtext);
    register_callback(collector_context, c"TextSubtext".as_ptr(), Raylib_TextSubtext);
    register_callback(collector_context, c"text_subtext".as_ptr(), Raylib_TextSubtext);
    register_callback(collector_context, c"Raylib::TextRemoveSpaces".as_ptr(), Raylib_TextRemoveSpaces);
    register_callback(collector_context, c"Raylib::text_remove_spaces".as_ptr(), Raylib_TextRemoveSpaces);
    register_callback(collector_context, c"raylib::TextRemoveSpaces".as_ptr(), Raylib_TextRemoveSpaces);
    register_callback(collector_context, c"raylib::text_remove_spaces".as_ptr(), Raylib_TextRemoveSpaces);
    register_callback(collector_context, c"TextRemoveSpaces".as_ptr(), Raylib_TextRemoveSpaces);
    register_callback(collector_context, c"text_remove_spaces".as_ptr(), Raylib_TextRemoveSpaces);
    register_callback(collector_context, c"Raylib::GetTextBetween".as_ptr(), Raylib_GetTextBetween);
    register_callback(collector_context, c"Raylib::get_text_between".as_ptr(), Raylib_GetTextBetween);
    register_callback(collector_context, c"raylib::GetTextBetween".as_ptr(), Raylib_GetTextBetween);
    register_callback(collector_context, c"raylib::get_text_between".as_ptr(), Raylib_GetTextBetween);
    register_callback(collector_context, c"GetTextBetween".as_ptr(), Raylib_GetTextBetween);
    register_callback(collector_context, c"get_text_between".as_ptr(), Raylib_GetTextBetween);
    register_callback(collector_context, c"Raylib::TextReplace".as_ptr(), Raylib_TextReplace);
    register_callback(collector_context, c"Raylib::text_replace".as_ptr(), Raylib_TextReplace);
    register_callback(collector_context, c"raylib::TextReplace".as_ptr(), Raylib_TextReplace);
    register_callback(collector_context, c"raylib::text_replace".as_ptr(), Raylib_TextReplace);
    register_callback(collector_context, c"TextReplace".as_ptr(), Raylib_TextReplace);
    register_callback(collector_context, c"text_replace".as_ptr(), Raylib_TextReplace);
    register_callback(collector_context, c"Raylib::TextReplaceAlloc".as_ptr(), Raylib_TextReplaceAlloc);
    register_callback(collector_context, c"Raylib::text_replace_alloc".as_ptr(), Raylib_TextReplaceAlloc);
    register_callback(collector_context, c"raylib::TextReplaceAlloc".as_ptr(), Raylib_TextReplaceAlloc);
    register_callback(collector_context, c"raylib::text_replace_alloc".as_ptr(), Raylib_TextReplaceAlloc);
    register_callback(collector_context, c"TextReplaceAlloc".as_ptr(), Raylib_TextReplaceAlloc);
    register_callback(collector_context, c"text_replace_alloc".as_ptr(), Raylib_TextReplaceAlloc);
    register_callback(collector_context, c"Raylib::TextReplaceBetween".as_ptr(), Raylib_TextReplaceBetween);
    register_callback(collector_context, c"Raylib::text_replace_between".as_ptr(), Raylib_TextReplaceBetween);
    register_callback(collector_context, c"raylib::TextReplaceBetween".as_ptr(), Raylib_TextReplaceBetween);
    register_callback(collector_context, c"raylib::text_replace_between".as_ptr(), Raylib_TextReplaceBetween);
    register_callback(collector_context, c"TextReplaceBetween".as_ptr(), Raylib_TextReplaceBetween);
    register_callback(collector_context, c"text_replace_between".as_ptr(), Raylib_TextReplaceBetween);
    register_callback(collector_context, c"Raylib::TextReplaceBetweenAlloc".as_ptr(), Raylib_TextReplaceBetweenAlloc);
    register_callback(collector_context, c"Raylib::text_replace_between_alloc".as_ptr(), Raylib_TextReplaceBetweenAlloc);
    register_callback(collector_context, c"raylib::TextReplaceBetweenAlloc".as_ptr(), Raylib_TextReplaceBetweenAlloc);
    register_callback(collector_context, c"raylib::text_replace_between_alloc".as_ptr(), Raylib_TextReplaceBetweenAlloc);
    register_callback(collector_context, c"TextReplaceBetweenAlloc".as_ptr(), Raylib_TextReplaceBetweenAlloc);
    register_callback(collector_context, c"text_replace_between_alloc".as_ptr(), Raylib_TextReplaceBetweenAlloc);
    register_callback(collector_context, c"Raylib::TextInsert".as_ptr(), Raylib_TextInsert);
    register_callback(collector_context, c"Raylib::text_insert".as_ptr(), Raylib_TextInsert);
    register_callback(collector_context, c"raylib::TextInsert".as_ptr(), Raylib_TextInsert);
    register_callback(collector_context, c"raylib::text_insert".as_ptr(), Raylib_TextInsert);
    register_callback(collector_context, c"TextInsert".as_ptr(), Raylib_TextInsert);
    register_callback(collector_context, c"text_insert".as_ptr(), Raylib_TextInsert);
    register_callback(collector_context, c"Raylib::TextInsertAlloc".as_ptr(), Raylib_TextInsertAlloc);
    register_callback(collector_context, c"Raylib::text_insert_alloc".as_ptr(), Raylib_TextInsertAlloc);
    register_callback(collector_context, c"raylib::TextInsertAlloc".as_ptr(), Raylib_TextInsertAlloc);
    register_callback(collector_context, c"raylib::text_insert_alloc".as_ptr(), Raylib_TextInsertAlloc);
    register_callback(collector_context, c"TextInsertAlloc".as_ptr(), Raylib_TextInsertAlloc);
    register_callback(collector_context, c"text_insert_alloc".as_ptr(), Raylib_TextInsertAlloc);
    register_callback(collector_context, c"Raylib::TextJoin".as_ptr(), Raylib_TextJoin);
    register_callback(collector_context, c"Raylib::text_join".as_ptr(), Raylib_TextJoin);
    register_callback(collector_context, c"raylib::TextJoin".as_ptr(), Raylib_TextJoin);
    register_callback(collector_context, c"raylib::text_join".as_ptr(), Raylib_TextJoin);
    register_callback(collector_context, c"TextJoin".as_ptr(), Raylib_TextJoin);
    register_callback(collector_context, c"text_join".as_ptr(), Raylib_TextJoin);
    register_callback(collector_context, c"Raylib::TextSplit".as_ptr(), Raylib_TextSplit);
    register_callback(collector_context, c"Raylib::text_split".as_ptr(), Raylib_TextSplit);
    register_callback(collector_context, c"raylib::TextSplit".as_ptr(), Raylib_TextSplit);
    register_callback(collector_context, c"raylib::text_split".as_ptr(), Raylib_TextSplit);
    register_callback(collector_context, c"TextSplit".as_ptr(), Raylib_TextSplit);
    register_callback(collector_context, c"text_split".as_ptr(), Raylib_TextSplit);
    register_callback(collector_context, c"Raylib::TextAppend".as_ptr(), Raylib_TextAppend);
    register_callback(collector_context, c"Raylib::text_append".as_ptr(), Raylib_TextAppend);
    register_callback(collector_context, c"raylib::TextAppend".as_ptr(), Raylib_TextAppend);
    register_callback(collector_context, c"raylib::text_append".as_ptr(), Raylib_TextAppend);
    register_callback(collector_context, c"TextAppend".as_ptr(), Raylib_TextAppend);
    register_callback(collector_context, c"text_append".as_ptr(), Raylib_TextAppend);
    register_callback(collector_context, c"Raylib::TextFindIndex".as_ptr(), Raylib_TextFindIndex);
    register_callback(collector_context, c"Raylib::text_find_index".as_ptr(), Raylib_TextFindIndex);
    register_callback(collector_context, c"raylib::TextFindIndex".as_ptr(), Raylib_TextFindIndex);
    register_callback(collector_context, c"raylib::text_find_index".as_ptr(), Raylib_TextFindIndex);
    register_callback(collector_context, c"TextFindIndex".as_ptr(), Raylib_TextFindIndex);
    register_callback(collector_context, c"text_find_index".as_ptr(), Raylib_TextFindIndex);
    register_callback(collector_context, c"Raylib::TextToUpper".as_ptr(), Raylib_TextToUpper);
    register_callback(collector_context, c"Raylib::text_to_upper".as_ptr(), Raylib_TextToUpper);
    register_callback(collector_context, c"raylib::TextToUpper".as_ptr(), Raylib_TextToUpper);
    register_callback(collector_context, c"raylib::text_to_upper".as_ptr(), Raylib_TextToUpper);
    register_callback(collector_context, c"TextToUpper".as_ptr(), Raylib_TextToUpper);
    register_callback(collector_context, c"text_to_upper".as_ptr(), Raylib_TextToUpper);
    register_callback(collector_context, c"Raylib::TextToLower".as_ptr(), Raylib_TextToLower);
    register_callback(collector_context, c"Raylib::text_to_lower".as_ptr(), Raylib_TextToLower);
    register_callback(collector_context, c"raylib::TextToLower".as_ptr(), Raylib_TextToLower);
    register_callback(collector_context, c"raylib::text_to_lower".as_ptr(), Raylib_TextToLower);
    register_callback(collector_context, c"TextToLower".as_ptr(), Raylib_TextToLower);
    register_callback(collector_context, c"text_to_lower".as_ptr(), Raylib_TextToLower);
    register_callback(collector_context, c"Raylib::TextToPascal".as_ptr(), Raylib_TextToPascal);
    register_callback(collector_context, c"Raylib::text_to_pascal".as_ptr(), Raylib_TextToPascal);
    register_callback(collector_context, c"raylib::TextToPascal".as_ptr(), Raylib_TextToPascal);
    register_callback(collector_context, c"raylib::text_to_pascal".as_ptr(), Raylib_TextToPascal);
    register_callback(collector_context, c"TextToPascal".as_ptr(), Raylib_TextToPascal);
    register_callback(collector_context, c"text_to_pascal".as_ptr(), Raylib_TextToPascal);
    register_callback(collector_context, c"Raylib::TextToSnake".as_ptr(), Raylib_TextToSnake);
    register_callback(collector_context, c"Raylib::text_to_snake".as_ptr(), Raylib_TextToSnake);
    register_callback(collector_context, c"raylib::TextToSnake".as_ptr(), Raylib_TextToSnake);
    register_callback(collector_context, c"raylib::text_to_snake".as_ptr(), Raylib_TextToSnake);
    register_callback(collector_context, c"TextToSnake".as_ptr(), Raylib_TextToSnake);
    register_callback(collector_context, c"text_to_snake".as_ptr(), Raylib_TextToSnake);
    register_callback(collector_context, c"Raylib::TextToCamel".as_ptr(), Raylib_TextToCamel);
    register_callback(collector_context, c"Raylib::text_to_camel".as_ptr(), Raylib_TextToCamel);
    register_callback(collector_context, c"raylib::TextToCamel".as_ptr(), Raylib_TextToCamel);
    register_callback(collector_context, c"raylib::text_to_camel".as_ptr(), Raylib_TextToCamel);
    register_callback(collector_context, c"TextToCamel".as_ptr(), Raylib_TextToCamel);
    register_callback(collector_context, c"text_to_camel".as_ptr(), Raylib_TextToCamel);
    register_callback(collector_context, c"Raylib::TextToInteger".as_ptr(), Raylib_TextToInteger);
    register_callback(collector_context, c"Raylib::text_to_integer".as_ptr(), Raylib_TextToInteger);
    register_callback(collector_context, c"raylib::TextToInteger".as_ptr(), Raylib_TextToInteger);
    register_callback(collector_context, c"raylib::text_to_integer".as_ptr(), Raylib_TextToInteger);
    register_callback(collector_context, c"TextToInteger".as_ptr(), Raylib_TextToInteger);
    register_callback(collector_context, c"text_to_integer".as_ptr(), Raylib_TextToInteger);
    register_callback(collector_context, c"Raylib::TextToFloat".as_ptr(), Raylib_TextToFloat);
    register_callback(collector_context, c"Raylib::text_to_float".as_ptr(), Raylib_TextToFloat);
    register_callback(collector_context, c"raylib::TextToFloat".as_ptr(), Raylib_TextToFloat);
    register_callback(collector_context, c"raylib::text_to_float".as_ptr(), Raylib_TextToFloat);
    register_callback(collector_context, c"TextToFloat".as_ptr(), Raylib_TextToFloat);
    register_callback(collector_context, c"text_to_float".as_ptr(), Raylib_TextToFloat);
    register_callback(collector_context, c"Raylib::DrawLine3D".as_ptr(), Raylib_DrawLine3D);
    register_callback(collector_context, c"Raylib::draw_line3d".as_ptr(), Raylib_DrawLine3D);
    register_callback(collector_context, c"raylib::DrawLine3D".as_ptr(), Raylib_DrawLine3D);
    register_callback(collector_context, c"raylib::draw_line3d".as_ptr(), Raylib_DrawLine3D);
    register_callback(collector_context, c"DrawLine3D".as_ptr(), Raylib_DrawLine3D);
    register_callback(collector_context, c"draw_line3d".as_ptr(), Raylib_DrawLine3D);
    register_callback(collector_context, c"Raylib::DrawPoint3D".as_ptr(), Raylib_DrawPoint3D);
    register_callback(collector_context, c"Raylib::draw_point3d".as_ptr(), Raylib_DrawPoint3D);
    register_callback(collector_context, c"raylib::DrawPoint3D".as_ptr(), Raylib_DrawPoint3D);
    register_callback(collector_context, c"raylib::draw_point3d".as_ptr(), Raylib_DrawPoint3D);
    register_callback(collector_context, c"DrawPoint3D".as_ptr(), Raylib_DrawPoint3D);
    register_callback(collector_context, c"draw_point3d".as_ptr(), Raylib_DrawPoint3D);
    register_callback(collector_context, c"Raylib::DrawCircle3D".as_ptr(), Raylib_DrawCircle3D);
    register_callback(collector_context, c"Raylib::draw_circle3d".as_ptr(), Raylib_DrawCircle3D);
    register_callback(collector_context, c"raylib::DrawCircle3D".as_ptr(), Raylib_DrawCircle3D);
    register_callback(collector_context, c"raylib::draw_circle3d".as_ptr(), Raylib_DrawCircle3D);
    register_callback(collector_context, c"DrawCircle3D".as_ptr(), Raylib_DrawCircle3D);
    register_callback(collector_context, c"draw_circle3d".as_ptr(), Raylib_DrawCircle3D);
    register_callback(collector_context, c"Raylib::DrawTriangle3D".as_ptr(), Raylib_DrawTriangle3D);
    register_callback(collector_context, c"Raylib::draw_triangle3d".as_ptr(), Raylib_DrawTriangle3D);
    register_callback(collector_context, c"raylib::DrawTriangle3D".as_ptr(), Raylib_DrawTriangle3D);
    register_callback(collector_context, c"raylib::draw_triangle3d".as_ptr(), Raylib_DrawTriangle3D);
    register_callback(collector_context, c"DrawTriangle3D".as_ptr(), Raylib_DrawTriangle3D);
    register_callback(collector_context, c"draw_triangle3d".as_ptr(), Raylib_DrawTriangle3D);
    register_callback(collector_context, c"Raylib::DrawTriangleStrip3D".as_ptr(), Raylib_DrawTriangleStrip3D);
    register_callback(collector_context, c"Raylib::draw_triangle_strip3d".as_ptr(), Raylib_DrawTriangleStrip3D);
    register_callback(collector_context, c"raylib::DrawTriangleStrip3D".as_ptr(), Raylib_DrawTriangleStrip3D);
    register_callback(collector_context, c"raylib::draw_triangle_strip3d".as_ptr(), Raylib_DrawTriangleStrip3D);
    register_callback(collector_context, c"DrawTriangleStrip3D".as_ptr(), Raylib_DrawTriangleStrip3D);
    register_callback(collector_context, c"draw_triangle_strip3d".as_ptr(), Raylib_DrawTriangleStrip3D);
    register_callback(collector_context, c"Raylib::DrawCube".as_ptr(), Raylib_DrawCube);
    register_callback(collector_context, c"Raylib::draw_cube".as_ptr(), Raylib_DrawCube);
    register_callback(collector_context, c"raylib::DrawCube".as_ptr(), Raylib_DrawCube);
    register_callback(collector_context, c"raylib::draw_cube".as_ptr(), Raylib_DrawCube);
    register_callback(collector_context, c"DrawCube".as_ptr(), Raylib_DrawCube);
    register_callback(collector_context, c"draw_cube".as_ptr(), Raylib_DrawCube);
    register_callback(collector_context, c"Raylib::DrawCubeV".as_ptr(), Raylib_DrawCubeV);
    register_callback(collector_context, c"Raylib::draw_cube_v".as_ptr(), Raylib_DrawCubeV);
    register_callback(collector_context, c"raylib::DrawCubeV".as_ptr(), Raylib_DrawCubeV);
    register_callback(collector_context, c"raylib::draw_cube_v".as_ptr(), Raylib_DrawCubeV);
    register_callback(collector_context, c"DrawCubeV".as_ptr(), Raylib_DrawCubeV);
    register_callback(collector_context, c"draw_cube_v".as_ptr(), Raylib_DrawCubeV);
    register_callback(collector_context, c"Raylib::DrawCubeWires".as_ptr(), Raylib_DrawCubeWires);
    register_callback(collector_context, c"Raylib::draw_cube_wires".as_ptr(), Raylib_DrawCubeWires);
    register_callback(collector_context, c"raylib::DrawCubeWires".as_ptr(), Raylib_DrawCubeWires);
    register_callback(collector_context, c"raylib::draw_cube_wires".as_ptr(), Raylib_DrawCubeWires);
    register_callback(collector_context, c"DrawCubeWires".as_ptr(), Raylib_DrawCubeWires);
    register_callback(collector_context, c"draw_cube_wires".as_ptr(), Raylib_DrawCubeWires);
    register_callback(collector_context, c"Raylib::DrawCubeWiresV".as_ptr(), Raylib_DrawCubeWiresV);
    register_callback(collector_context, c"Raylib::draw_cube_wires_v".as_ptr(), Raylib_DrawCubeWiresV);
    register_callback(collector_context, c"raylib::DrawCubeWiresV".as_ptr(), Raylib_DrawCubeWiresV);
    register_callback(collector_context, c"raylib::draw_cube_wires_v".as_ptr(), Raylib_DrawCubeWiresV);
    register_callback(collector_context, c"DrawCubeWiresV".as_ptr(), Raylib_DrawCubeWiresV);
    register_callback(collector_context, c"draw_cube_wires_v".as_ptr(), Raylib_DrawCubeWiresV);
    register_callback(collector_context, c"Raylib::DrawSphere".as_ptr(), Raylib_DrawSphere);
    register_callback(collector_context, c"Raylib::draw_sphere".as_ptr(), Raylib_DrawSphere);
    register_callback(collector_context, c"raylib::DrawSphere".as_ptr(), Raylib_DrawSphere);
    register_callback(collector_context, c"raylib::draw_sphere".as_ptr(), Raylib_DrawSphere);
    register_callback(collector_context, c"DrawSphere".as_ptr(), Raylib_DrawSphere);
    register_callback(collector_context, c"draw_sphere".as_ptr(), Raylib_DrawSphere);
    register_callback(collector_context, c"Raylib::DrawSphereEx".as_ptr(), Raylib_DrawSphereEx);
    register_callback(collector_context, c"Raylib::draw_sphere_ex".as_ptr(), Raylib_DrawSphereEx);
    register_callback(collector_context, c"raylib::DrawSphereEx".as_ptr(), Raylib_DrawSphereEx);
    register_callback(collector_context, c"raylib::draw_sphere_ex".as_ptr(), Raylib_DrawSphereEx);
    register_callback(collector_context, c"DrawSphereEx".as_ptr(), Raylib_DrawSphereEx);
    register_callback(collector_context, c"draw_sphere_ex".as_ptr(), Raylib_DrawSphereEx);
    register_callback(collector_context, c"Raylib::DrawSphereWires".as_ptr(), Raylib_DrawSphereWires);
    register_callback(collector_context, c"Raylib::draw_sphere_wires".as_ptr(), Raylib_DrawSphereWires);
    register_callback(collector_context, c"raylib::DrawSphereWires".as_ptr(), Raylib_DrawSphereWires);
    register_callback(collector_context, c"raylib::draw_sphere_wires".as_ptr(), Raylib_DrawSphereWires);
    register_callback(collector_context, c"DrawSphereWires".as_ptr(), Raylib_DrawSphereWires);
    register_callback(collector_context, c"draw_sphere_wires".as_ptr(), Raylib_DrawSphereWires);
    register_callback(collector_context, c"Raylib::DrawCylinder".as_ptr(), Raylib_DrawCylinder);
    register_callback(collector_context, c"Raylib::draw_cylinder".as_ptr(), Raylib_DrawCylinder);
    register_callback(collector_context, c"raylib::DrawCylinder".as_ptr(), Raylib_DrawCylinder);
    register_callback(collector_context, c"raylib::draw_cylinder".as_ptr(), Raylib_DrawCylinder);
    register_callback(collector_context, c"DrawCylinder".as_ptr(), Raylib_DrawCylinder);
    register_callback(collector_context, c"draw_cylinder".as_ptr(), Raylib_DrawCylinder);
    register_callback(collector_context, c"Raylib::DrawCylinderEx".as_ptr(), Raylib_DrawCylinderEx);
    register_callback(collector_context, c"Raylib::draw_cylinder_ex".as_ptr(), Raylib_DrawCylinderEx);
    register_callback(collector_context, c"raylib::DrawCylinderEx".as_ptr(), Raylib_DrawCylinderEx);
    register_callback(collector_context, c"raylib::draw_cylinder_ex".as_ptr(), Raylib_DrawCylinderEx);
    register_callback(collector_context, c"DrawCylinderEx".as_ptr(), Raylib_DrawCylinderEx);
    register_callback(collector_context, c"draw_cylinder_ex".as_ptr(), Raylib_DrawCylinderEx);
    register_callback(collector_context, c"Raylib::DrawCylinderWires".as_ptr(), Raylib_DrawCylinderWires);
    register_callback(collector_context, c"Raylib::draw_cylinder_wires".as_ptr(), Raylib_DrawCylinderWires);
    register_callback(collector_context, c"raylib::DrawCylinderWires".as_ptr(), Raylib_DrawCylinderWires);
    register_callback(collector_context, c"raylib::draw_cylinder_wires".as_ptr(), Raylib_DrawCylinderWires);
    register_callback(collector_context, c"DrawCylinderWires".as_ptr(), Raylib_DrawCylinderWires);
    register_callback(collector_context, c"draw_cylinder_wires".as_ptr(), Raylib_DrawCylinderWires);
    register_callback(collector_context, c"Raylib::DrawCylinderWiresEx".as_ptr(), Raylib_DrawCylinderWiresEx);
    register_callback(collector_context, c"Raylib::draw_cylinder_wires_ex".as_ptr(), Raylib_DrawCylinderWiresEx);
    register_callback(collector_context, c"raylib::DrawCylinderWiresEx".as_ptr(), Raylib_DrawCylinderWiresEx);
    register_callback(collector_context, c"raylib::draw_cylinder_wires_ex".as_ptr(), Raylib_DrawCylinderWiresEx);
    register_callback(collector_context, c"DrawCylinderWiresEx".as_ptr(), Raylib_DrawCylinderWiresEx);
    register_callback(collector_context, c"draw_cylinder_wires_ex".as_ptr(), Raylib_DrawCylinderWiresEx);
    register_callback(collector_context, c"Raylib::DrawCapsule".as_ptr(), Raylib_DrawCapsule);
    register_callback(collector_context, c"Raylib::draw_capsule".as_ptr(), Raylib_DrawCapsule);
    register_callback(collector_context, c"raylib::DrawCapsule".as_ptr(), Raylib_DrawCapsule);
    register_callback(collector_context, c"raylib::draw_capsule".as_ptr(), Raylib_DrawCapsule);
    register_callback(collector_context, c"DrawCapsule".as_ptr(), Raylib_DrawCapsule);
    register_callback(collector_context, c"draw_capsule".as_ptr(), Raylib_DrawCapsule);
    register_callback(collector_context, c"Raylib::DrawCapsuleWires".as_ptr(), Raylib_DrawCapsuleWires);
    register_callback(collector_context, c"Raylib::draw_capsule_wires".as_ptr(), Raylib_DrawCapsuleWires);
    register_callback(collector_context, c"raylib::DrawCapsuleWires".as_ptr(), Raylib_DrawCapsuleWires);
    register_callback(collector_context, c"raylib::draw_capsule_wires".as_ptr(), Raylib_DrawCapsuleWires);
    register_callback(collector_context, c"DrawCapsuleWires".as_ptr(), Raylib_DrawCapsuleWires);
    register_callback(collector_context, c"draw_capsule_wires".as_ptr(), Raylib_DrawCapsuleWires);
    register_callback(collector_context, c"Raylib::DrawPlane".as_ptr(), Raylib_DrawPlane);
    register_callback(collector_context, c"Raylib::draw_plane".as_ptr(), Raylib_DrawPlane);
    register_callback(collector_context, c"raylib::DrawPlane".as_ptr(), Raylib_DrawPlane);
    register_callback(collector_context, c"raylib::draw_plane".as_ptr(), Raylib_DrawPlane);
    register_callback(collector_context, c"DrawPlane".as_ptr(), Raylib_DrawPlane);
    register_callback(collector_context, c"draw_plane".as_ptr(), Raylib_DrawPlane);
    register_callback(collector_context, c"Raylib::DrawRay".as_ptr(), Raylib_DrawRay);
    register_callback(collector_context, c"Raylib::draw_ray".as_ptr(), Raylib_DrawRay);
    register_callback(collector_context, c"raylib::DrawRay".as_ptr(), Raylib_DrawRay);
    register_callback(collector_context, c"raylib::draw_ray".as_ptr(), Raylib_DrawRay);
    register_callback(collector_context, c"DrawRay".as_ptr(), Raylib_DrawRay);
    register_callback(collector_context, c"draw_ray".as_ptr(), Raylib_DrawRay);
    register_callback(collector_context, c"Raylib::DrawGrid".as_ptr(), Raylib_DrawGrid);
    register_callback(collector_context, c"Raylib::draw_grid".as_ptr(), Raylib_DrawGrid);
    register_callback(collector_context, c"raylib::DrawGrid".as_ptr(), Raylib_DrawGrid);
    register_callback(collector_context, c"raylib::draw_grid".as_ptr(), Raylib_DrawGrid);
    register_callback(collector_context, c"DrawGrid".as_ptr(), Raylib_DrawGrid);
    register_callback(collector_context, c"draw_grid".as_ptr(), Raylib_DrawGrid);
    register_callback(collector_context, c"Raylib::LoadModel".as_ptr(), Raylib_LoadModel);
    register_callback(collector_context, c"Raylib::load_model".as_ptr(), Raylib_LoadModel);
    register_callback(collector_context, c"raylib::LoadModel".as_ptr(), Raylib_LoadModel);
    register_callback(collector_context, c"raylib::load_model".as_ptr(), Raylib_LoadModel);
    register_callback(collector_context, c"LoadModel".as_ptr(), Raylib_LoadModel);
    register_callback(collector_context, c"load_model".as_ptr(), Raylib_LoadModel);
    register_callback(collector_context, c"Raylib::LoadModelFromMesh".as_ptr(), Raylib_LoadModelFromMesh);
    register_callback(collector_context, c"Raylib::load_model_from_mesh".as_ptr(), Raylib_LoadModelFromMesh);
    register_callback(collector_context, c"raylib::LoadModelFromMesh".as_ptr(), Raylib_LoadModelFromMesh);
    register_callback(collector_context, c"raylib::load_model_from_mesh".as_ptr(), Raylib_LoadModelFromMesh);
    register_callback(collector_context, c"LoadModelFromMesh".as_ptr(), Raylib_LoadModelFromMesh);
    register_callback(collector_context, c"load_model_from_mesh".as_ptr(), Raylib_LoadModelFromMesh);
    register_callback(collector_context, c"Raylib::IsModelValid".as_ptr(), Raylib_IsModelValid);
    register_callback(collector_context, c"Raylib::is_model_valid".as_ptr(), Raylib_IsModelValid);
    register_callback(collector_context, c"raylib::IsModelValid".as_ptr(), Raylib_IsModelValid);
    register_callback(collector_context, c"raylib::is_model_valid".as_ptr(), Raylib_IsModelValid);
    register_callback(collector_context, c"IsModelValid".as_ptr(), Raylib_IsModelValid);
    register_callback(collector_context, c"is_model_valid".as_ptr(), Raylib_IsModelValid);
    register_callback(collector_context, c"Raylib::UnloadModel".as_ptr(), Raylib_UnloadModel);
    register_callback(collector_context, c"Raylib::unload_model".as_ptr(), Raylib_UnloadModel);
    register_callback(collector_context, c"raylib::UnloadModel".as_ptr(), Raylib_UnloadModel);
    register_callback(collector_context, c"raylib::unload_model".as_ptr(), Raylib_UnloadModel);
    register_callback(collector_context, c"UnloadModel".as_ptr(), Raylib_UnloadModel);
    register_callback(collector_context, c"unload_model".as_ptr(), Raylib_UnloadModel);
    register_callback(collector_context, c"Raylib::GetModelBoundingBox".as_ptr(), Raylib_GetModelBoundingBox);
    register_callback(collector_context, c"Raylib::get_model_bounding_box".as_ptr(), Raylib_GetModelBoundingBox);
    register_callback(collector_context, c"raylib::GetModelBoundingBox".as_ptr(), Raylib_GetModelBoundingBox);
    register_callback(collector_context, c"raylib::get_model_bounding_box".as_ptr(), Raylib_GetModelBoundingBox);
    register_callback(collector_context, c"GetModelBoundingBox".as_ptr(), Raylib_GetModelBoundingBox);
    register_callback(collector_context, c"get_model_bounding_box".as_ptr(), Raylib_GetModelBoundingBox);
    register_callback(collector_context, c"Raylib::DrawModel".as_ptr(), Raylib_DrawModel);
    register_callback(collector_context, c"Raylib::draw_model".as_ptr(), Raylib_DrawModel);
    register_callback(collector_context, c"raylib::DrawModel".as_ptr(), Raylib_DrawModel);
    register_callback(collector_context, c"raylib::draw_model".as_ptr(), Raylib_DrawModel);
    register_callback(collector_context, c"DrawModel".as_ptr(), Raylib_DrawModel);
    register_callback(collector_context, c"draw_model".as_ptr(), Raylib_DrawModel);
    register_callback(collector_context, c"Raylib::DrawModelEx".as_ptr(), Raylib_DrawModelEx);
    register_callback(collector_context, c"Raylib::draw_model_ex".as_ptr(), Raylib_DrawModelEx);
    register_callback(collector_context, c"raylib::DrawModelEx".as_ptr(), Raylib_DrawModelEx);
    register_callback(collector_context, c"raylib::draw_model_ex".as_ptr(), Raylib_DrawModelEx);
    register_callback(collector_context, c"DrawModelEx".as_ptr(), Raylib_DrawModelEx);
    register_callback(collector_context, c"draw_model_ex".as_ptr(), Raylib_DrawModelEx);
    register_callback(collector_context, c"Raylib::DrawModelWires".as_ptr(), Raylib_DrawModelWires);
    register_callback(collector_context, c"Raylib::draw_model_wires".as_ptr(), Raylib_DrawModelWires);
    register_callback(collector_context, c"raylib::DrawModelWires".as_ptr(), Raylib_DrawModelWires);
    register_callback(collector_context, c"raylib::draw_model_wires".as_ptr(), Raylib_DrawModelWires);
    register_callback(collector_context, c"DrawModelWires".as_ptr(), Raylib_DrawModelWires);
    register_callback(collector_context, c"draw_model_wires".as_ptr(), Raylib_DrawModelWires);
    register_callback(collector_context, c"Raylib::DrawModelWiresEx".as_ptr(), Raylib_DrawModelWiresEx);
    register_callback(collector_context, c"Raylib::draw_model_wires_ex".as_ptr(), Raylib_DrawModelWiresEx);
    register_callback(collector_context, c"raylib::DrawModelWiresEx".as_ptr(), Raylib_DrawModelWiresEx);
    register_callback(collector_context, c"raylib::draw_model_wires_ex".as_ptr(), Raylib_DrawModelWiresEx);
    register_callback(collector_context, c"DrawModelWiresEx".as_ptr(), Raylib_DrawModelWiresEx);
    register_callback(collector_context, c"draw_model_wires_ex".as_ptr(), Raylib_DrawModelWiresEx);
    register_callback(collector_context, c"Raylib::DrawBoundingBox".as_ptr(), Raylib_DrawBoundingBox);
    register_callback(collector_context, c"Raylib::draw_bounding_box".as_ptr(), Raylib_DrawBoundingBox);
    register_callback(collector_context, c"raylib::DrawBoundingBox".as_ptr(), Raylib_DrawBoundingBox);
    register_callback(collector_context, c"raylib::draw_bounding_box".as_ptr(), Raylib_DrawBoundingBox);
    register_callback(collector_context, c"DrawBoundingBox".as_ptr(), Raylib_DrawBoundingBox);
    register_callback(collector_context, c"draw_bounding_box".as_ptr(), Raylib_DrawBoundingBox);
    register_callback(collector_context, c"Raylib::DrawBillboard".as_ptr(), Raylib_DrawBillboard);
    register_callback(collector_context, c"Raylib::draw_billboard".as_ptr(), Raylib_DrawBillboard);
    register_callback(collector_context, c"raylib::DrawBillboard".as_ptr(), Raylib_DrawBillboard);
    register_callback(collector_context, c"raylib::draw_billboard".as_ptr(), Raylib_DrawBillboard);
    register_callback(collector_context, c"DrawBillboard".as_ptr(), Raylib_DrawBillboard);
    register_callback(collector_context, c"draw_billboard".as_ptr(), Raylib_DrawBillboard);
    register_callback(collector_context, c"Raylib::DrawBillboardRec".as_ptr(), Raylib_DrawBillboardRec);
    register_callback(collector_context, c"Raylib::draw_billboard_rec".as_ptr(), Raylib_DrawBillboardRec);
    register_callback(collector_context, c"raylib::DrawBillboardRec".as_ptr(), Raylib_DrawBillboardRec);
    register_callback(collector_context, c"raylib::draw_billboard_rec".as_ptr(), Raylib_DrawBillboardRec);
    register_callback(collector_context, c"DrawBillboardRec".as_ptr(), Raylib_DrawBillboardRec);
    register_callback(collector_context, c"draw_billboard_rec".as_ptr(), Raylib_DrawBillboardRec);
    register_callback(collector_context, c"Raylib::DrawBillboardPro".as_ptr(), Raylib_DrawBillboardPro);
    register_callback(collector_context, c"Raylib::draw_billboard_pro".as_ptr(), Raylib_DrawBillboardPro);
    register_callback(collector_context, c"raylib::DrawBillboardPro".as_ptr(), Raylib_DrawBillboardPro);
    register_callback(collector_context, c"raylib::draw_billboard_pro".as_ptr(), Raylib_DrawBillboardPro);
    register_callback(collector_context, c"DrawBillboardPro".as_ptr(), Raylib_DrawBillboardPro);
    register_callback(collector_context, c"draw_billboard_pro".as_ptr(), Raylib_DrawBillboardPro);
    register_callback(collector_context, c"Raylib::UploadMesh".as_ptr(), Raylib_UploadMesh);
    register_callback(collector_context, c"Raylib::upload_mesh".as_ptr(), Raylib_UploadMesh);
    register_callback(collector_context, c"raylib::UploadMesh".as_ptr(), Raylib_UploadMesh);
    register_callback(collector_context, c"raylib::upload_mesh".as_ptr(), Raylib_UploadMesh);
    register_callback(collector_context, c"UploadMesh".as_ptr(), Raylib_UploadMesh);
    register_callback(collector_context, c"upload_mesh".as_ptr(), Raylib_UploadMesh);
    register_callback(collector_context, c"Raylib::UpdateMeshBuffer".as_ptr(), Raylib_UpdateMeshBuffer);
    register_callback(collector_context, c"Raylib::update_mesh_buffer".as_ptr(), Raylib_UpdateMeshBuffer);
    register_callback(collector_context, c"raylib::UpdateMeshBuffer".as_ptr(), Raylib_UpdateMeshBuffer);
    register_callback(collector_context, c"raylib::update_mesh_buffer".as_ptr(), Raylib_UpdateMeshBuffer);
    register_callback(collector_context, c"UpdateMeshBuffer".as_ptr(), Raylib_UpdateMeshBuffer);
    register_callback(collector_context, c"update_mesh_buffer".as_ptr(), Raylib_UpdateMeshBuffer);
    register_callback(collector_context, c"Raylib::UnloadMesh".as_ptr(), Raylib_UnloadMesh);
    register_callback(collector_context, c"Raylib::unload_mesh".as_ptr(), Raylib_UnloadMesh);
    register_callback(collector_context, c"raylib::UnloadMesh".as_ptr(), Raylib_UnloadMesh);
    register_callback(collector_context, c"raylib::unload_mesh".as_ptr(), Raylib_UnloadMesh);
    register_callback(collector_context, c"UnloadMesh".as_ptr(), Raylib_UnloadMesh);
    register_callback(collector_context, c"unload_mesh".as_ptr(), Raylib_UnloadMesh);
    register_callback(collector_context, c"Raylib::DrawMesh".as_ptr(), Raylib_DrawMesh);
    register_callback(collector_context, c"Raylib::draw_mesh".as_ptr(), Raylib_DrawMesh);
    register_callback(collector_context, c"raylib::DrawMesh".as_ptr(), Raylib_DrawMesh);
    register_callback(collector_context, c"raylib::draw_mesh".as_ptr(), Raylib_DrawMesh);
    register_callback(collector_context, c"DrawMesh".as_ptr(), Raylib_DrawMesh);
    register_callback(collector_context, c"draw_mesh".as_ptr(), Raylib_DrawMesh);
    register_callback(collector_context, c"Raylib::DrawMeshInstanced".as_ptr(), Raylib_DrawMeshInstanced);
    register_callback(collector_context, c"Raylib::draw_mesh_instanced".as_ptr(), Raylib_DrawMeshInstanced);
    register_callback(collector_context, c"raylib::DrawMeshInstanced".as_ptr(), Raylib_DrawMeshInstanced);
    register_callback(collector_context, c"raylib::draw_mesh_instanced".as_ptr(), Raylib_DrawMeshInstanced);
    register_callback(collector_context, c"DrawMeshInstanced".as_ptr(), Raylib_DrawMeshInstanced);
    register_callback(collector_context, c"draw_mesh_instanced".as_ptr(), Raylib_DrawMeshInstanced);
    register_callback(collector_context, c"Raylib::GetMeshBoundingBox".as_ptr(), Raylib_GetMeshBoundingBox);
    register_callback(collector_context, c"Raylib::get_mesh_bounding_box".as_ptr(), Raylib_GetMeshBoundingBox);
    register_callback(collector_context, c"raylib::GetMeshBoundingBox".as_ptr(), Raylib_GetMeshBoundingBox);
    register_callback(collector_context, c"raylib::get_mesh_bounding_box".as_ptr(), Raylib_GetMeshBoundingBox);
    register_callback(collector_context, c"GetMeshBoundingBox".as_ptr(), Raylib_GetMeshBoundingBox);
    register_callback(collector_context, c"get_mesh_bounding_box".as_ptr(), Raylib_GetMeshBoundingBox);
    register_callback(collector_context, c"Raylib::GenMeshTangents".as_ptr(), Raylib_GenMeshTangents);
    register_callback(collector_context, c"Raylib::gen_mesh_tangents".as_ptr(), Raylib_GenMeshTangents);
    register_callback(collector_context, c"raylib::GenMeshTangents".as_ptr(), Raylib_GenMeshTangents);
    register_callback(collector_context, c"raylib::gen_mesh_tangents".as_ptr(), Raylib_GenMeshTangents);
    register_callback(collector_context, c"GenMeshTangents".as_ptr(), Raylib_GenMeshTangents);
    register_callback(collector_context, c"gen_mesh_tangents".as_ptr(), Raylib_GenMeshTangents);
    register_callback(collector_context, c"Raylib::ExportMesh".as_ptr(), Raylib_ExportMesh);
    register_callback(collector_context, c"Raylib::export_mesh".as_ptr(), Raylib_ExportMesh);
    register_callback(collector_context, c"raylib::ExportMesh".as_ptr(), Raylib_ExportMesh);
    register_callback(collector_context, c"raylib::export_mesh".as_ptr(), Raylib_ExportMesh);
    register_callback(collector_context, c"ExportMesh".as_ptr(), Raylib_ExportMesh);
    register_callback(collector_context, c"export_mesh".as_ptr(), Raylib_ExportMesh);
    register_callback(collector_context, c"Raylib::ExportMeshAsCode".as_ptr(), Raylib_ExportMeshAsCode);
    register_callback(collector_context, c"Raylib::export_mesh_as_code".as_ptr(), Raylib_ExportMeshAsCode);
    register_callback(collector_context, c"raylib::ExportMeshAsCode".as_ptr(), Raylib_ExportMeshAsCode);
    register_callback(collector_context, c"raylib::export_mesh_as_code".as_ptr(), Raylib_ExportMeshAsCode);
    register_callback(collector_context, c"ExportMeshAsCode".as_ptr(), Raylib_ExportMeshAsCode);
    register_callback(collector_context, c"export_mesh_as_code".as_ptr(), Raylib_ExportMeshAsCode);
    register_callback(collector_context, c"Raylib::GenMeshPoly".as_ptr(), Raylib_GenMeshPoly);
    register_callback(collector_context, c"Raylib::gen_mesh_poly".as_ptr(), Raylib_GenMeshPoly);
    register_callback(collector_context, c"raylib::GenMeshPoly".as_ptr(), Raylib_GenMeshPoly);
    register_callback(collector_context, c"raylib::gen_mesh_poly".as_ptr(), Raylib_GenMeshPoly);
    register_callback(collector_context, c"GenMeshPoly".as_ptr(), Raylib_GenMeshPoly);
    register_callback(collector_context, c"gen_mesh_poly".as_ptr(), Raylib_GenMeshPoly);
    register_callback(collector_context, c"Raylib::GenMeshPlane".as_ptr(), Raylib_GenMeshPlane);
    register_callback(collector_context, c"Raylib::gen_mesh_plane".as_ptr(), Raylib_GenMeshPlane);
    register_callback(collector_context, c"raylib::GenMeshPlane".as_ptr(), Raylib_GenMeshPlane);
    register_callback(collector_context, c"raylib::gen_mesh_plane".as_ptr(), Raylib_GenMeshPlane);
    register_callback(collector_context, c"GenMeshPlane".as_ptr(), Raylib_GenMeshPlane);
    register_callback(collector_context, c"gen_mesh_plane".as_ptr(), Raylib_GenMeshPlane);
    register_callback(collector_context, c"Raylib::GenMeshCube".as_ptr(), Raylib_GenMeshCube);
    register_callback(collector_context, c"Raylib::gen_mesh_cube".as_ptr(), Raylib_GenMeshCube);
    register_callback(collector_context, c"raylib::GenMeshCube".as_ptr(), Raylib_GenMeshCube);
    register_callback(collector_context, c"raylib::gen_mesh_cube".as_ptr(), Raylib_GenMeshCube);
    register_callback(collector_context, c"GenMeshCube".as_ptr(), Raylib_GenMeshCube);
    register_callback(collector_context, c"gen_mesh_cube".as_ptr(), Raylib_GenMeshCube);
    register_callback(collector_context, c"Raylib::GenMeshSphere".as_ptr(), Raylib_GenMeshSphere);
    register_callback(collector_context, c"Raylib::gen_mesh_sphere".as_ptr(), Raylib_GenMeshSphere);
    register_callback(collector_context, c"raylib::GenMeshSphere".as_ptr(), Raylib_GenMeshSphere);
    register_callback(collector_context, c"raylib::gen_mesh_sphere".as_ptr(), Raylib_GenMeshSphere);
    register_callback(collector_context, c"GenMeshSphere".as_ptr(), Raylib_GenMeshSphere);
    register_callback(collector_context, c"gen_mesh_sphere".as_ptr(), Raylib_GenMeshSphere);
    register_callback(collector_context, c"Raylib::GenMeshHemiSphere".as_ptr(), Raylib_GenMeshHemiSphere);
    register_callback(collector_context, c"Raylib::gen_mesh_hemi_sphere".as_ptr(), Raylib_GenMeshHemiSphere);
    register_callback(collector_context, c"raylib::GenMeshHemiSphere".as_ptr(), Raylib_GenMeshHemiSphere);
    register_callback(collector_context, c"raylib::gen_mesh_hemi_sphere".as_ptr(), Raylib_GenMeshHemiSphere);
    register_callback(collector_context, c"GenMeshHemiSphere".as_ptr(), Raylib_GenMeshHemiSphere);
    register_callback(collector_context, c"gen_mesh_hemi_sphere".as_ptr(), Raylib_GenMeshHemiSphere);
    register_callback(collector_context, c"Raylib::GenMeshCylinder".as_ptr(), Raylib_GenMeshCylinder);
    register_callback(collector_context, c"Raylib::gen_mesh_cylinder".as_ptr(), Raylib_GenMeshCylinder);
    register_callback(collector_context, c"raylib::GenMeshCylinder".as_ptr(), Raylib_GenMeshCylinder);
    register_callback(collector_context, c"raylib::gen_mesh_cylinder".as_ptr(), Raylib_GenMeshCylinder);
    register_callback(collector_context, c"GenMeshCylinder".as_ptr(), Raylib_GenMeshCylinder);
    register_callback(collector_context, c"gen_mesh_cylinder".as_ptr(), Raylib_GenMeshCylinder);
    register_callback(collector_context, c"Raylib::GenMeshCone".as_ptr(), Raylib_GenMeshCone);
    register_callback(collector_context, c"Raylib::gen_mesh_cone".as_ptr(), Raylib_GenMeshCone);
    register_callback(collector_context, c"raylib::GenMeshCone".as_ptr(), Raylib_GenMeshCone);
    register_callback(collector_context, c"raylib::gen_mesh_cone".as_ptr(), Raylib_GenMeshCone);
    register_callback(collector_context, c"GenMeshCone".as_ptr(), Raylib_GenMeshCone);
    register_callback(collector_context, c"gen_mesh_cone".as_ptr(), Raylib_GenMeshCone);
    register_callback(collector_context, c"Raylib::GenMeshTorus".as_ptr(), Raylib_GenMeshTorus);
    register_callback(collector_context, c"Raylib::gen_mesh_torus".as_ptr(), Raylib_GenMeshTorus);
    register_callback(collector_context, c"raylib::GenMeshTorus".as_ptr(), Raylib_GenMeshTorus);
    register_callback(collector_context, c"raylib::gen_mesh_torus".as_ptr(), Raylib_GenMeshTorus);
    register_callback(collector_context, c"GenMeshTorus".as_ptr(), Raylib_GenMeshTorus);
    register_callback(collector_context, c"gen_mesh_torus".as_ptr(), Raylib_GenMeshTorus);
    register_callback(collector_context, c"Raylib::GenMeshKnot".as_ptr(), Raylib_GenMeshKnot);
    register_callback(collector_context, c"Raylib::gen_mesh_knot".as_ptr(), Raylib_GenMeshKnot);
    register_callback(collector_context, c"raylib::GenMeshKnot".as_ptr(), Raylib_GenMeshKnot);
    register_callback(collector_context, c"raylib::gen_mesh_knot".as_ptr(), Raylib_GenMeshKnot);
    register_callback(collector_context, c"GenMeshKnot".as_ptr(), Raylib_GenMeshKnot);
    register_callback(collector_context, c"gen_mesh_knot".as_ptr(), Raylib_GenMeshKnot);
    register_callback(collector_context, c"Raylib::GenMeshHeightmap".as_ptr(), Raylib_GenMeshHeightmap);
    register_callback(collector_context, c"Raylib::gen_mesh_heightmap".as_ptr(), Raylib_GenMeshHeightmap);
    register_callback(collector_context, c"raylib::GenMeshHeightmap".as_ptr(), Raylib_GenMeshHeightmap);
    register_callback(collector_context, c"raylib::gen_mesh_heightmap".as_ptr(), Raylib_GenMeshHeightmap);
    register_callback(collector_context, c"GenMeshHeightmap".as_ptr(), Raylib_GenMeshHeightmap);
    register_callback(collector_context, c"gen_mesh_heightmap".as_ptr(), Raylib_GenMeshHeightmap);
    register_callback(collector_context, c"Raylib::GenMeshCubicmap".as_ptr(), Raylib_GenMeshCubicmap);
    register_callback(collector_context, c"Raylib::gen_mesh_cubicmap".as_ptr(), Raylib_GenMeshCubicmap);
    register_callback(collector_context, c"raylib::GenMeshCubicmap".as_ptr(), Raylib_GenMeshCubicmap);
    register_callback(collector_context, c"raylib::gen_mesh_cubicmap".as_ptr(), Raylib_GenMeshCubicmap);
    register_callback(collector_context, c"GenMeshCubicmap".as_ptr(), Raylib_GenMeshCubicmap);
    register_callback(collector_context, c"gen_mesh_cubicmap".as_ptr(), Raylib_GenMeshCubicmap);
    register_callback(collector_context, c"Raylib::LoadMaterials".as_ptr(), Raylib_LoadMaterials);
    register_callback(collector_context, c"Raylib::load_materials".as_ptr(), Raylib_LoadMaterials);
    register_callback(collector_context, c"raylib::LoadMaterials".as_ptr(), Raylib_LoadMaterials);
    register_callback(collector_context, c"raylib::load_materials".as_ptr(), Raylib_LoadMaterials);
    register_callback(collector_context, c"LoadMaterials".as_ptr(), Raylib_LoadMaterials);
    register_callback(collector_context, c"load_materials".as_ptr(), Raylib_LoadMaterials);
    register_callback(collector_context, c"Raylib::LoadMaterialDefault".as_ptr(), Raylib_LoadMaterialDefault);
    register_callback(collector_context, c"Raylib::load_material_default".as_ptr(), Raylib_LoadMaterialDefault);
    register_callback(collector_context, c"raylib::LoadMaterialDefault".as_ptr(), Raylib_LoadMaterialDefault);
    register_callback(collector_context, c"raylib::load_material_default".as_ptr(), Raylib_LoadMaterialDefault);
    register_callback(collector_context, c"LoadMaterialDefault".as_ptr(), Raylib_LoadMaterialDefault);
    register_callback(collector_context, c"load_material_default".as_ptr(), Raylib_LoadMaterialDefault);
    register_callback(collector_context, c"Raylib::IsMaterialValid".as_ptr(), Raylib_IsMaterialValid);
    register_callback(collector_context, c"Raylib::is_material_valid".as_ptr(), Raylib_IsMaterialValid);
    register_callback(collector_context, c"raylib::IsMaterialValid".as_ptr(), Raylib_IsMaterialValid);
    register_callback(collector_context, c"raylib::is_material_valid".as_ptr(), Raylib_IsMaterialValid);
    register_callback(collector_context, c"IsMaterialValid".as_ptr(), Raylib_IsMaterialValid);
    register_callback(collector_context, c"is_material_valid".as_ptr(), Raylib_IsMaterialValid);
    register_callback(collector_context, c"Raylib::UnloadMaterial".as_ptr(), Raylib_UnloadMaterial);
    register_callback(collector_context, c"Raylib::unload_material".as_ptr(), Raylib_UnloadMaterial);
    register_callback(collector_context, c"raylib::UnloadMaterial".as_ptr(), Raylib_UnloadMaterial);
    register_callback(collector_context, c"raylib::unload_material".as_ptr(), Raylib_UnloadMaterial);
    register_callback(collector_context, c"UnloadMaterial".as_ptr(), Raylib_UnloadMaterial);
    register_callback(collector_context, c"unload_material".as_ptr(), Raylib_UnloadMaterial);
    register_callback(collector_context, c"Raylib::SetMaterialTexture".as_ptr(), Raylib_SetMaterialTexture);
    register_callback(collector_context, c"Raylib::set_material_texture".as_ptr(), Raylib_SetMaterialTexture);
    register_callback(collector_context, c"raylib::SetMaterialTexture".as_ptr(), Raylib_SetMaterialTexture);
    register_callback(collector_context, c"raylib::set_material_texture".as_ptr(), Raylib_SetMaterialTexture);
    register_callback(collector_context, c"SetMaterialTexture".as_ptr(), Raylib_SetMaterialTexture);
    register_callback(collector_context, c"set_material_texture".as_ptr(), Raylib_SetMaterialTexture);
    register_callback(collector_context, c"Raylib::SetModelMeshMaterial".as_ptr(), Raylib_SetModelMeshMaterial);
    register_callback(collector_context, c"Raylib::set_model_mesh_material".as_ptr(), Raylib_SetModelMeshMaterial);
    register_callback(collector_context, c"raylib::SetModelMeshMaterial".as_ptr(), Raylib_SetModelMeshMaterial);
    register_callback(collector_context, c"raylib::set_model_mesh_material".as_ptr(), Raylib_SetModelMeshMaterial);
    register_callback(collector_context, c"SetModelMeshMaterial".as_ptr(), Raylib_SetModelMeshMaterial);
    register_callback(collector_context, c"set_model_mesh_material".as_ptr(), Raylib_SetModelMeshMaterial);
    register_callback(collector_context, c"Raylib::LoadModelAnimations".as_ptr(), Raylib_LoadModelAnimations);
    register_callback(collector_context, c"Raylib::load_model_animations".as_ptr(), Raylib_LoadModelAnimations);
    register_callback(collector_context, c"raylib::LoadModelAnimations".as_ptr(), Raylib_LoadModelAnimations);
    register_callback(collector_context, c"raylib::load_model_animations".as_ptr(), Raylib_LoadModelAnimations);
    register_callback(collector_context, c"LoadModelAnimations".as_ptr(), Raylib_LoadModelAnimations);
    register_callback(collector_context, c"load_model_animations".as_ptr(), Raylib_LoadModelAnimations);
    register_callback(collector_context, c"Raylib::UpdateModelAnimation".as_ptr(), Raylib_UpdateModelAnimation);
    register_callback(collector_context, c"Raylib::update_model_animation".as_ptr(), Raylib_UpdateModelAnimation);
    register_callback(collector_context, c"raylib::UpdateModelAnimation".as_ptr(), Raylib_UpdateModelAnimation);
    register_callback(collector_context, c"raylib::update_model_animation".as_ptr(), Raylib_UpdateModelAnimation);
    register_callback(collector_context, c"UpdateModelAnimation".as_ptr(), Raylib_UpdateModelAnimation);
    register_callback(collector_context, c"update_model_animation".as_ptr(), Raylib_UpdateModelAnimation);
    register_callback(collector_context, c"Raylib::UpdateModelAnimationEx".as_ptr(), Raylib_UpdateModelAnimationEx);
    register_callback(collector_context, c"Raylib::update_model_animation_ex".as_ptr(), Raylib_UpdateModelAnimationEx);
    register_callback(collector_context, c"raylib::UpdateModelAnimationEx".as_ptr(), Raylib_UpdateModelAnimationEx);
    register_callback(collector_context, c"raylib::update_model_animation_ex".as_ptr(), Raylib_UpdateModelAnimationEx);
    register_callback(collector_context, c"UpdateModelAnimationEx".as_ptr(), Raylib_UpdateModelAnimationEx);
    register_callback(collector_context, c"update_model_animation_ex".as_ptr(), Raylib_UpdateModelAnimationEx);
    register_callback(collector_context, c"Raylib::UnloadModelAnimations".as_ptr(), Raylib_UnloadModelAnimations);
    register_callback(collector_context, c"Raylib::unload_model_animations".as_ptr(), Raylib_UnloadModelAnimations);
    register_callback(collector_context, c"raylib::UnloadModelAnimations".as_ptr(), Raylib_UnloadModelAnimations);
    register_callback(collector_context, c"raylib::unload_model_animations".as_ptr(), Raylib_UnloadModelAnimations);
    register_callback(collector_context, c"UnloadModelAnimations".as_ptr(), Raylib_UnloadModelAnimations);
    register_callback(collector_context, c"unload_model_animations".as_ptr(), Raylib_UnloadModelAnimations);
    register_callback(collector_context, c"Raylib::IsModelAnimationValid".as_ptr(), Raylib_IsModelAnimationValid);
    register_callback(collector_context, c"Raylib::is_model_animation_valid".as_ptr(), Raylib_IsModelAnimationValid);
    register_callback(collector_context, c"raylib::IsModelAnimationValid".as_ptr(), Raylib_IsModelAnimationValid);
    register_callback(collector_context, c"raylib::is_model_animation_valid".as_ptr(), Raylib_IsModelAnimationValid);
    register_callback(collector_context, c"IsModelAnimationValid".as_ptr(), Raylib_IsModelAnimationValid);
    register_callback(collector_context, c"is_model_animation_valid".as_ptr(), Raylib_IsModelAnimationValid);
    register_callback(collector_context, c"Raylib::CheckCollisionSpheres".as_ptr(), Raylib_CheckCollisionSpheres);
    register_callback(collector_context, c"Raylib::check_collision_spheres".as_ptr(), Raylib_CheckCollisionSpheres);
    register_callback(collector_context, c"raylib::CheckCollisionSpheres".as_ptr(), Raylib_CheckCollisionSpheres);
    register_callback(collector_context, c"raylib::check_collision_spheres".as_ptr(), Raylib_CheckCollisionSpheres);
    register_callback(collector_context, c"CheckCollisionSpheres".as_ptr(), Raylib_CheckCollisionSpheres);
    register_callback(collector_context, c"check_collision_spheres".as_ptr(), Raylib_CheckCollisionSpheres);
    register_callback(collector_context, c"Raylib::CheckCollisionBoxes".as_ptr(), Raylib_CheckCollisionBoxes);
    register_callback(collector_context, c"Raylib::check_collision_boxes".as_ptr(), Raylib_CheckCollisionBoxes);
    register_callback(collector_context, c"raylib::CheckCollisionBoxes".as_ptr(), Raylib_CheckCollisionBoxes);
    register_callback(collector_context, c"raylib::check_collision_boxes".as_ptr(), Raylib_CheckCollisionBoxes);
    register_callback(collector_context, c"CheckCollisionBoxes".as_ptr(), Raylib_CheckCollisionBoxes);
    register_callback(collector_context, c"check_collision_boxes".as_ptr(), Raylib_CheckCollisionBoxes);
    register_callback(collector_context, c"Raylib::CheckCollisionBoxSphere".as_ptr(), Raylib_CheckCollisionBoxSphere);
    register_callback(collector_context, c"Raylib::check_collision_box_sphere".as_ptr(), Raylib_CheckCollisionBoxSphere);
    register_callback(collector_context, c"raylib::CheckCollisionBoxSphere".as_ptr(), Raylib_CheckCollisionBoxSphere);
    register_callback(collector_context, c"raylib::check_collision_box_sphere".as_ptr(), Raylib_CheckCollisionBoxSphere);
    register_callback(collector_context, c"CheckCollisionBoxSphere".as_ptr(), Raylib_CheckCollisionBoxSphere);
    register_callback(collector_context, c"check_collision_box_sphere".as_ptr(), Raylib_CheckCollisionBoxSphere);
    register_callback(collector_context, c"Raylib::GetRayCollisionSphere".as_ptr(), Raylib_GetRayCollisionSphere);
    register_callback(collector_context, c"Raylib::get_ray_collision_sphere".as_ptr(), Raylib_GetRayCollisionSphere);
    register_callback(collector_context, c"raylib::GetRayCollisionSphere".as_ptr(), Raylib_GetRayCollisionSphere);
    register_callback(collector_context, c"raylib::get_ray_collision_sphere".as_ptr(), Raylib_GetRayCollisionSphere);
    register_callback(collector_context, c"GetRayCollisionSphere".as_ptr(), Raylib_GetRayCollisionSphere);
    register_callback(collector_context, c"get_ray_collision_sphere".as_ptr(), Raylib_GetRayCollisionSphere);
    register_callback(collector_context, c"Raylib::GetRayCollisionBox".as_ptr(), Raylib_GetRayCollisionBox);
    register_callback(collector_context, c"Raylib::get_ray_collision_box".as_ptr(), Raylib_GetRayCollisionBox);
    register_callback(collector_context, c"raylib::GetRayCollisionBox".as_ptr(), Raylib_GetRayCollisionBox);
    register_callback(collector_context, c"raylib::get_ray_collision_box".as_ptr(), Raylib_GetRayCollisionBox);
    register_callback(collector_context, c"GetRayCollisionBox".as_ptr(), Raylib_GetRayCollisionBox);
    register_callback(collector_context, c"get_ray_collision_box".as_ptr(), Raylib_GetRayCollisionBox);
    register_callback(collector_context, c"Raylib::GetRayCollisionMesh".as_ptr(), Raylib_GetRayCollisionMesh);
    register_callback(collector_context, c"Raylib::get_ray_collision_mesh".as_ptr(), Raylib_GetRayCollisionMesh);
    register_callback(collector_context, c"raylib::GetRayCollisionMesh".as_ptr(), Raylib_GetRayCollisionMesh);
    register_callback(collector_context, c"raylib::get_ray_collision_mesh".as_ptr(), Raylib_GetRayCollisionMesh);
    register_callback(collector_context, c"GetRayCollisionMesh".as_ptr(), Raylib_GetRayCollisionMesh);
    register_callback(collector_context, c"get_ray_collision_mesh".as_ptr(), Raylib_GetRayCollisionMesh);
    register_callback(collector_context, c"Raylib::GetRayCollisionTriangle".as_ptr(), Raylib_GetRayCollisionTriangle);
    register_callback(collector_context, c"Raylib::get_ray_collision_triangle".as_ptr(), Raylib_GetRayCollisionTriangle);
    register_callback(collector_context, c"raylib::GetRayCollisionTriangle".as_ptr(), Raylib_GetRayCollisionTriangle);
    register_callback(collector_context, c"raylib::get_ray_collision_triangle".as_ptr(), Raylib_GetRayCollisionTriangle);
    register_callback(collector_context, c"GetRayCollisionTriangle".as_ptr(), Raylib_GetRayCollisionTriangle);
    register_callback(collector_context, c"get_ray_collision_triangle".as_ptr(), Raylib_GetRayCollisionTriangle);
    register_callback(collector_context, c"Raylib::GetRayCollisionQuad".as_ptr(), Raylib_GetRayCollisionQuad);
    register_callback(collector_context, c"Raylib::get_ray_collision_quad".as_ptr(), Raylib_GetRayCollisionQuad);
    register_callback(collector_context, c"raylib::GetRayCollisionQuad".as_ptr(), Raylib_GetRayCollisionQuad);
    register_callback(collector_context, c"raylib::get_ray_collision_quad".as_ptr(), Raylib_GetRayCollisionQuad);
    register_callback(collector_context, c"GetRayCollisionQuad".as_ptr(), Raylib_GetRayCollisionQuad);
    register_callback(collector_context, c"get_ray_collision_quad".as_ptr(), Raylib_GetRayCollisionQuad);
    register_callback(collector_context, c"Raylib::InitAudioDevice".as_ptr(), Raylib_InitAudioDevice);
    register_callback(collector_context, c"Raylib::init_audio_device".as_ptr(), Raylib_InitAudioDevice);
    register_callback(collector_context, c"raylib::InitAudioDevice".as_ptr(), Raylib_InitAudioDevice);
    register_callback(collector_context, c"raylib::init_audio_device".as_ptr(), Raylib_InitAudioDevice);
    register_callback(collector_context, c"InitAudioDevice".as_ptr(), Raylib_InitAudioDevice);
    register_callback(collector_context, c"init_audio_device".as_ptr(), Raylib_InitAudioDevice);
    register_callback(collector_context, c"Raylib::CloseAudioDevice".as_ptr(), Raylib_CloseAudioDevice);
    register_callback(collector_context, c"Raylib::close_audio_device".as_ptr(), Raylib_CloseAudioDevice);
    register_callback(collector_context, c"raylib::CloseAudioDevice".as_ptr(), Raylib_CloseAudioDevice);
    register_callback(collector_context, c"raylib::close_audio_device".as_ptr(), Raylib_CloseAudioDevice);
    register_callback(collector_context, c"CloseAudioDevice".as_ptr(), Raylib_CloseAudioDevice);
    register_callback(collector_context, c"close_audio_device".as_ptr(), Raylib_CloseAudioDevice);
    register_callback(collector_context, c"Raylib::IsAudioDeviceReady".as_ptr(), Raylib_IsAudioDeviceReady);
    register_callback(collector_context, c"Raylib::is_audio_device_ready".as_ptr(), Raylib_IsAudioDeviceReady);
    register_callback(collector_context, c"raylib::IsAudioDeviceReady".as_ptr(), Raylib_IsAudioDeviceReady);
    register_callback(collector_context, c"raylib::is_audio_device_ready".as_ptr(), Raylib_IsAudioDeviceReady);
    register_callback(collector_context, c"IsAudioDeviceReady".as_ptr(), Raylib_IsAudioDeviceReady);
    register_callback(collector_context, c"is_audio_device_ready".as_ptr(), Raylib_IsAudioDeviceReady);
    register_callback(collector_context, c"Raylib::SetMasterVolume".as_ptr(), Raylib_SetMasterVolume);
    register_callback(collector_context, c"Raylib::set_master_volume".as_ptr(), Raylib_SetMasterVolume);
    register_callback(collector_context, c"raylib::SetMasterVolume".as_ptr(), Raylib_SetMasterVolume);
    register_callback(collector_context, c"raylib::set_master_volume".as_ptr(), Raylib_SetMasterVolume);
    register_callback(collector_context, c"SetMasterVolume".as_ptr(), Raylib_SetMasterVolume);
    register_callback(collector_context, c"set_master_volume".as_ptr(), Raylib_SetMasterVolume);
    register_callback(collector_context, c"Raylib::GetMasterVolume".as_ptr(), Raylib_GetMasterVolume);
    register_callback(collector_context, c"Raylib::get_master_volume".as_ptr(), Raylib_GetMasterVolume);
    register_callback(collector_context, c"raylib::GetMasterVolume".as_ptr(), Raylib_GetMasterVolume);
    register_callback(collector_context, c"raylib::get_master_volume".as_ptr(), Raylib_GetMasterVolume);
    register_callback(collector_context, c"GetMasterVolume".as_ptr(), Raylib_GetMasterVolume);
    register_callback(collector_context, c"get_master_volume".as_ptr(), Raylib_GetMasterVolume);
    register_callback(collector_context, c"Raylib::LoadWave".as_ptr(), Raylib_LoadWave);
    register_callback(collector_context, c"Raylib::load_wave".as_ptr(), Raylib_LoadWave);
    register_callback(collector_context, c"raylib::LoadWave".as_ptr(), Raylib_LoadWave);
    register_callback(collector_context, c"raylib::load_wave".as_ptr(), Raylib_LoadWave);
    register_callback(collector_context, c"LoadWave".as_ptr(), Raylib_LoadWave);
    register_callback(collector_context, c"load_wave".as_ptr(), Raylib_LoadWave);
    register_callback(collector_context, c"Raylib::LoadWaveFromMemory".as_ptr(), Raylib_LoadWaveFromMemory);
    register_callback(collector_context, c"Raylib::load_wave_from_memory".as_ptr(), Raylib_LoadWaveFromMemory);
    register_callback(collector_context, c"raylib::LoadWaveFromMemory".as_ptr(), Raylib_LoadWaveFromMemory);
    register_callback(collector_context, c"raylib::load_wave_from_memory".as_ptr(), Raylib_LoadWaveFromMemory);
    register_callback(collector_context, c"LoadWaveFromMemory".as_ptr(), Raylib_LoadWaveFromMemory);
    register_callback(collector_context, c"load_wave_from_memory".as_ptr(), Raylib_LoadWaveFromMemory);
    register_callback(collector_context, c"Raylib::IsWaveValid".as_ptr(), Raylib_IsWaveValid);
    register_callback(collector_context, c"Raylib::is_wave_valid".as_ptr(), Raylib_IsWaveValid);
    register_callback(collector_context, c"raylib::IsWaveValid".as_ptr(), Raylib_IsWaveValid);
    register_callback(collector_context, c"raylib::is_wave_valid".as_ptr(), Raylib_IsWaveValid);
    register_callback(collector_context, c"IsWaveValid".as_ptr(), Raylib_IsWaveValid);
    register_callback(collector_context, c"is_wave_valid".as_ptr(), Raylib_IsWaveValid);
    register_callback(collector_context, c"Raylib::LoadSound".as_ptr(), Raylib_LoadSound);
    register_callback(collector_context, c"Raylib::load_sound".as_ptr(), Raylib_LoadSound);
    register_callback(collector_context, c"raylib::LoadSound".as_ptr(), Raylib_LoadSound);
    register_callback(collector_context, c"raylib::load_sound".as_ptr(), Raylib_LoadSound);
    register_callback(collector_context, c"LoadSound".as_ptr(), Raylib_LoadSound);
    register_callback(collector_context, c"load_sound".as_ptr(), Raylib_LoadSound);
    register_callback(collector_context, c"Raylib::LoadSoundFromWave".as_ptr(), Raylib_LoadSoundFromWave);
    register_callback(collector_context, c"Raylib::load_sound_from_wave".as_ptr(), Raylib_LoadSoundFromWave);
    register_callback(collector_context, c"raylib::LoadSoundFromWave".as_ptr(), Raylib_LoadSoundFromWave);
    register_callback(collector_context, c"raylib::load_sound_from_wave".as_ptr(), Raylib_LoadSoundFromWave);
    register_callback(collector_context, c"LoadSoundFromWave".as_ptr(), Raylib_LoadSoundFromWave);
    register_callback(collector_context, c"load_sound_from_wave".as_ptr(), Raylib_LoadSoundFromWave);
    register_callback(collector_context, c"Raylib::LoadSoundAlias".as_ptr(), Raylib_LoadSoundAlias);
    register_callback(collector_context, c"Raylib::load_sound_alias".as_ptr(), Raylib_LoadSoundAlias);
    register_callback(collector_context, c"raylib::LoadSoundAlias".as_ptr(), Raylib_LoadSoundAlias);
    register_callback(collector_context, c"raylib::load_sound_alias".as_ptr(), Raylib_LoadSoundAlias);
    register_callback(collector_context, c"LoadSoundAlias".as_ptr(), Raylib_LoadSoundAlias);
    register_callback(collector_context, c"load_sound_alias".as_ptr(), Raylib_LoadSoundAlias);
    register_callback(collector_context, c"Raylib::IsSoundValid".as_ptr(), Raylib_IsSoundValid);
    register_callback(collector_context, c"Raylib::is_sound_valid".as_ptr(), Raylib_IsSoundValid);
    register_callback(collector_context, c"raylib::IsSoundValid".as_ptr(), Raylib_IsSoundValid);
    register_callback(collector_context, c"raylib::is_sound_valid".as_ptr(), Raylib_IsSoundValid);
    register_callback(collector_context, c"IsSoundValid".as_ptr(), Raylib_IsSoundValid);
    register_callback(collector_context, c"is_sound_valid".as_ptr(), Raylib_IsSoundValid);
    register_callback(collector_context, c"Raylib::UpdateSound".as_ptr(), Raylib_UpdateSound);
    register_callback(collector_context, c"Raylib::update_sound".as_ptr(), Raylib_UpdateSound);
    register_callback(collector_context, c"raylib::UpdateSound".as_ptr(), Raylib_UpdateSound);
    register_callback(collector_context, c"raylib::update_sound".as_ptr(), Raylib_UpdateSound);
    register_callback(collector_context, c"UpdateSound".as_ptr(), Raylib_UpdateSound);
    register_callback(collector_context, c"update_sound".as_ptr(), Raylib_UpdateSound);
    register_callback(collector_context, c"Raylib::UnloadWave".as_ptr(), Raylib_UnloadWave);
    register_callback(collector_context, c"Raylib::unload_wave".as_ptr(), Raylib_UnloadWave);
    register_callback(collector_context, c"raylib::UnloadWave".as_ptr(), Raylib_UnloadWave);
    register_callback(collector_context, c"raylib::unload_wave".as_ptr(), Raylib_UnloadWave);
    register_callback(collector_context, c"UnloadWave".as_ptr(), Raylib_UnloadWave);
    register_callback(collector_context, c"unload_wave".as_ptr(), Raylib_UnloadWave);
    register_callback(collector_context, c"Raylib::UnloadSound".as_ptr(), Raylib_UnloadSound);
    register_callback(collector_context, c"Raylib::unload_sound".as_ptr(), Raylib_UnloadSound);
    register_callback(collector_context, c"raylib::UnloadSound".as_ptr(), Raylib_UnloadSound);
    register_callback(collector_context, c"raylib::unload_sound".as_ptr(), Raylib_UnloadSound);
    register_callback(collector_context, c"UnloadSound".as_ptr(), Raylib_UnloadSound);
    register_callback(collector_context, c"unload_sound".as_ptr(), Raylib_UnloadSound);
    register_callback(collector_context, c"Raylib::UnloadSoundAlias".as_ptr(), Raylib_UnloadSoundAlias);
    register_callback(collector_context, c"Raylib::unload_sound_alias".as_ptr(), Raylib_UnloadSoundAlias);
    register_callback(collector_context, c"raylib::UnloadSoundAlias".as_ptr(), Raylib_UnloadSoundAlias);
    register_callback(collector_context, c"raylib::unload_sound_alias".as_ptr(), Raylib_UnloadSoundAlias);
    register_callback(collector_context, c"UnloadSoundAlias".as_ptr(), Raylib_UnloadSoundAlias);
    register_callback(collector_context, c"unload_sound_alias".as_ptr(), Raylib_UnloadSoundAlias);
    register_callback(collector_context, c"Raylib::ExportWave".as_ptr(), Raylib_ExportWave);
    register_callback(collector_context, c"Raylib::export_wave".as_ptr(), Raylib_ExportWave);
    register_callback(collector_context, c"raylib::ExportWave".as_ptr(), Raylib_ExportWave);
    register_callback(collector_context, c"raylib::export_wave".as_ptr(), Raylib_ExportWave);
    register_callback(collector_context, c"ExportWave".as_ptr(), Raylib_ExportWave);
    register_callback(collector_context, c"export_wave".as_ptr(), Raylib_ExportWave);
    register_callback(collector_context, c"Raylib::ExportWaveAsCode".as_ptr(), Raylib_ExportWaveAsCode);
    register_callback(collector_context, c"Raylib::export_wave_as_code".as_ptr(), Raylib_ExportWaveAsCode);
    register_callback(collector_context, c"raylib::ExportWaveAsCode".as_ptr(), Raylib_ExportWaveAsCode);
    register_callback(collector_context, c"raylib::export_wave_as_code".as_ptr(), Raylib_ExportWaveAsCode);
    register_callback(collector_context, c"ExportWaveAsCode".as_ptr(), Raylib_ExportWaveAsCode);
    register_callback(collector_context, c"export_wave_as_code".as_ptr(), Raylib_ExportWaveAsCode);
    register_callback(collector_context, c"Raylib::PlaySound".as_ptr(), Raylib_PlaySound);
    register_callback(collector_context, c"Raylib::play_sound".as_ptr(), Raylib_PlaySound);
    register_callback(collector_context, c"raylib::PlaySound".as_ptr(), Raylib_PlaySound);
    register_callback(collector_context, c"raylib::play_sound".as_ptr(), Raylib_PlaySound);
    register_callback(collector_context, c"PlaySound".as_ptr(), Raylib_PlaySound);
    register_callback(collector_context, c"play_sound".as_ptr(), Raylib_PlaySound);
    register_callback(collector_context, c"Raylib::StopSound".as_ptr(), Raylib_StopSound);
    register_callback(collector_context, c"Raylib::stop_sound".as_ptr(), Raylib_StopSound);
    register_callback(collector_context, c"raylib::StopSound".as_ptr(), Raylib_StopSound);
    register_callback(collector_context, c"raylib::stop_sound".as_ptr(), Raylib_StopSound);
    register_callback(collector_context, c"StopSound".as_ptr(), Raylib_StopSound);
    register_callback(collector_context, c"stop_sound".as_ptr(), Raylib_StopSound);
    register_callback(collector_context, c"Raylib::PauseSound".as_ptr(), Raylib_PauseSound);
    register_callback(collector_context, c"Raylib::pause_sound".as_ptr(), Raylib_PauseSound);
    register_callback(collector_context, c"raylib::PauseSound".as_ptr(), Raylib_PauseSound);
    register_callback(collector_context, c"raylib::pause_sound".as_ptr(), Raylib_PauseSound);
    register_callback(collector_context, c"PauseSound".as_ptr(), Raylib_PauseSound);
    register_callback(collector_context, c"pause_sound".as_ptr(), Raylib_PauseSound);
    register_callback(collector_context, c"Raylib::ResumeSound".as_ptr(), Raylib_ResumeSound);
    register_callback(collector_context, c"Raylib::resume_sound".as_ptr(), Raylib_ResumeSound);
    register_callback(collector_context, c"raylib::ResumeSound".as_ptr(), Raylib_ResumeSound);
    register_callback(collector_context, c"raylib::resume_sound".as_ptr(), Raylib_ResumeSound);
    register_callback(collector_context, c"ResumeSound".as_ptr(), Raylib_ResumeSound);
    register_callback(collector_context, c"resume_sound".as_ptr(), Raylib_ResumeSound);
    register_callback(collector_context, c"Raylib::IsSoundPlaying".as_ptr(), Raylib_IsSoundPlaying);
    register_callback(collector_context, c"Raylib::is_sound_playing".as_ptr(), Raylib_IsSoundPlaying);
    register_callback(collector_context, c"raylib::IsSoundPlaying".as_ptr(), Raylib_IsSoundPlaying);
    register_callback(collector_context, c"raylib::is_sound_playing".as_ptr(), Raylib_IsSoundPlaying);
    register_callback(collector_context, c"IsSoundPlaying".as_ptr(), Raylib_IsSoundPlaying);
    register_callback(collector_context, c"is_sound_playing".as_ptr(), Raylib_IsSoundPlaying);
    register_callback(collector_context, c"Raylib::SetSoundVolume".as_ptr(), Raylib_SetSoundVolume);
    register_callback(collector_context, c"Raylib::set_sound_volume".as_ptr(), Raylib_SetSoundVolume);
    register_callback(collector_context, c"raylib::SetSoundVolume".as_ptr(), Raylib_SetSoundVolume);
    register_callback(collector_context, c"raylib::set_sound_volume".as_ptr(), Raylib_SetSoundVolume);
    register_callback(collector_context, c"SetSoundVolume".as_ptr(), Raylib_SetSoundVolume);
    register_callback(collector_context, c"set_sound_volume".as_ptr(), Raylib_SetSoundVolume);
    register_callback(collector_context, c"Raylib::SetSoundPitch".as_ptr(), Raylib_SetSoundPitch);
    register_callback(collector_context, c"Raylib::set_sound_pitch".as_ptr(), Raylib_SetSoundPitch);
    register_callback(collector_context, c"raylib::SetSoundPitch".as_ptr(), Raylib_SetSoundPitch);
    register_callback(collector_context, c"raylib::set_sound_pitch".as_ptr(), Raylib_SetSoundPitch);
    register_callback(collector_context, c"SetSoundPitch".as_ptr(), Raylib_SetSoundPitch);
    register_callback(collector_context, c"set_sound_pitch".as_ptr(), Raylib_SetSoundPitch);
    register_callback(collector_context, c"Raylib::SetSoundPan".as_ptr(), Raylib_SetSoundPan);
    register_callback(collector_context, c"Raylib::set_sound_pan".as_ptr(), Raylib_SetSoundPan);
    register_callback(collector_context, c"raylib::SetSoundPan".as_ptr(), Raylib_SetSoundPan);
    register_callback(collector_context, c"raylib::set_sound_pan".as_ptr(), Raylib_SetSoundPan);
    register_callback(collector_context, c"SetSoundPan".as_ptr(), Raylib_SetSoundPan);
    register_callback(collector_context, c"set_sound_pan".as_ptr(), Raylib_SetSoundPan);
    register_callback(collector_context, c"Raylib::WaveCopy".as_ptr(), Raylib_WaveCopy);
    register_callback(collector_context, c"Raylib::wave_copy".as_ptr(), Raylib_WaveCopy);
    register_callback(collector_context, c"raylib::WaveCopy".as_ptr(), Raylib_WaveCopy);
    register_callback(collector_context, c"raylib::wave_copy".as_ptr(), Raylib_WaveCopy);
    register_callback(collector_context, c"WaveCopy".as_ptr(), Raylib_WaveCopy);
    register_callback(collector_context, c"wave_copy".as_ptr(), Raylib_WaveCopy);
    register_callback(collector_context, c"Raylib::WaveCrop".as_ptr(), Raylib_WaveCrop);
    register_callback(collector_context, c"Raylib::wave_crop".as_ptr(), Raylib_WaveCrop);
    register_callback(collector_context, c"raylib::WaveCrop".as_ptr(), Raylib_WaveCrop);
    register_callback(collector_context, c"raylib::wave_crop".as_ptr(), Raylib_WaveCrop);
    register_callback(collector_context, c"WaveCrop".as_ptr(), Raylib_WaveCrop);
    register_callback(collector_context, c"wave_crop".as_ptr(), Raylib_WaveCrop);
    register_callback(collector_context, c"Raylib::WaveFormat".as_ptr(), Raylib_WaveFormat);
    register_callback(collector_context, c"Raylib::wave_format".as_ptr(), Raylib_WaveFormat);
    register_callback(collector_context, c"raylib::WaveFormat".as_ptr(), Raylib_WaveFormat);
    register_callback(collector_context, c"raylib::wave_format".as_ptr(), Raylib_WaveFormat);
    register_callback(collector_context, c"WaveFormat".as_ptr(), Raylib_WaveFormat);
    register_callback(collector_context, c"wave_format".as_ptr(), Raylib_WaveFormat);
    register_callback(collector_context, c"Raylib::LoadWaveSamples".as_ptr(), Raylib_LoadWaveSamples);
    register_callback(collector_context, c"Raylib::load_wave_samples".as_ptr(), Raylib_LoadWaveSamples);
    register_callback(collector_context, c"raylib::LoadWaveSamples".as_ptr(), Raylib_LoadWaveSamples);
    register_callback(collector_context, c"raylib::load_wave_samples".as_ptr(), Raylib_LoadWaveSamples);
    register_callback(collector_context, c"LoadWaveSamples".as_ptr(), Raylib_LoadWaveSamples);
    register_callback(collector_context, c"load_wave_samples".as_ptr(), Raylib_LoadWaveSamples);
    register_callback(collector_context, c"Raylib::UnloadWaveSamples".as_ptr(), Raylib_UnloadWaveSamples);
    register_callback(collector_context, c"Raylib::unload_wave_samples".as_ptr(), Raylib_UnloadWaveSamples);
    register_callback(collector_context, c"raylib::UnloadWaveSamples".as_ptr(), Raylib_UnloadWaveSamples);
    register_callback(collector_context, c"raylib::unload_wave_samples".as_ptr(), Raylib_UnloadWaveSamples);
    register_callback(collector_context, c"UnloadWaveSamples".as_ptr(), Raylib_UnloadWaveSamples);
    register_callback(collector_context, c"unload_wave_samples".as_ptr(), Raylib_UnloadWaveSamples);
    register_callback(collector_context, c"Raylib::LoadMusicStream".as_ptr(), Raylib_LoadMusicStream);
    register_callback(collector_context, c"Raylib::load_music_stream".as_ptr(), Raylib_LoadMusicStream);
    register_callback(collector_context, c"raylib::LoadMusicStream".as_ptr(), Raylib_LoadMusicStream);
    register_callback(collector_context, c"raylib::load_music_stream".as_ptr(), Raylib_LoadMusicStream);
    register_callback(collector_context, c"LoadMusicStream".as_ptr(), Raylib_LoadMusicStream);
    register_callback(collector_context, c"load_music_stream".as_ptr(), Raylib_LoadMusicStream);
    register_callback(collector_context, c"Raylib::LoadMusicStreamFromMemory".as_ptr(), Raylib_LoadMusicStreamFromMemory);
    register_callback(collector_context, c"Raylib::load_music_stream_from_memory".as_ptr(), Raylib_LoadMusicStreamFromMemory);
    register_callback(collector_context, c"raylib::LoadMusicStreamFromMemory".as_ptr(), Raylib_LoadMusicStreamFromMemory);
    register_callback(collector_context, c"raylib::load_music_stream_from_memory".as_ptr(), Raylib_LoadMusicStreamFromMemory);
    register_callback(collector_context, c"LoadMusicStreamFromMemory".as_ptr(), Raylib_LoadMusicStreamFromMemory);
    register_callback(collector_context, c"load_music_stream_from_memory".as_ptr(), Raylib_LoadMusicStreamFromMemory);
    register_callback(collector_context, c"Raylib::IsMusicValid".as_ptr(), Raylib_IsMusicValid);
    register_callback(collector_context, c"Raylib::is_music_valid".as_ptr(), Raylib_IsMusicValid);
    register_callback(collector_context, c"raylib::IsMusicValid".as_ptr(), Raylib_IsMusicValid);
    register_callback(collector_context, c"raylib::is_music_valid".as_ptr(), Raylib_IsMusicValid);
    register_callback(collector_context, c"IsMusicValid".as_ptr(), Raylib_IsMusicValid);
    register_callback(collector_context, c"is_music_valid".as_ptr(), Raylib_IsMusicValid);
    register_callback(collector_context, c"Raylib::UnloadMusicStream".as_ptr(), Raylib_UnloadMusicStream);
    register_callback(collector_context, c"Raylib::unload_music_stream".as_ptr(), Raylib_UnloadMusicStream);
    register_callback(collector_context, c"raylib::UnloadMusicStream".as_ptr(), Raylib_UnloadMusicStream);
    register_callback(collector_context, c"raylib::unload_music_stream".as_ptr(), Raylib_UnloadMusicStream);
    register_callback(collector_context, c"UnloadMusicStream".as_ptr(), Raylib_UnloadMusicStream);
    register_callback(collector_context, c"unload_music_stream".as_ptr(), Raylib_UnloadMusicStream);
    register_callback(collector_context, c"Raylib::PlayMusicStream".as_ptr(), Raylib_PlayMusicStream);
    register_callback(collector_context, c"Raylib::play_music_stream".as_ptr(), Raylib_PlayMusicStream);
    register_callback(collector_context, c"raylib::PlayMusicStream".as_ptr(), Raylib_PlayMusicStream);
    register_callback(collector_context, c"raylib::play_music_stream".as_ptr(), Raylib_PlayMusicStream);
    register_callback(collector_context, c"PlayMusicStream".as_ptr(), Raylib_PlayMusicStream);
    register_callback(collector_context, c"play_music_stream".as_ptr(), Raylib_PlayMusicStream);
    register_callback(collector_context, c"Raylib::IsMusicStreamPlaying".as_ptr(), Raylib_IsMusicStreamPlaying);
    register_callback(collector_context, c"Raylib::is_music_stream_playing".as_ptr(), Raylib_IsMusicStreamPlaying);
    register_callback(collector_context, c"raylib::IsMusicStreamPlaying".as_ptr(), Raylib_IsMusicStreamPlaying);
    register_callback(collector_context, c"raylib::is_music_stream_playing".as_ptr(), Raylib_IsMusicStreamPlaying);
    register_callback(collector_context, c"IsMusicStreamPlaying".as_ptr(), Raylib_IsMusicStreamPlaying);
    register_callback(collector_context, c"is_music_stream_playing".as_ptr(), Raylib_IsMusicStreamPlaying);
    register_callback(collector_context, c"Raylib::UpdateMusicStream".as_ptr(), Raylib_UpdateMusicStream);
    register_callback(collector_context, c"Raylib::update_music_stream".as_ptr(), Raylib_UpdateMusicStream);
    register_callback(collector_context, c"raylib::UpdateMusicStream".as_ptr(), Raylib_UpdateMusicStream);
    register_callback(collector_context, c"raylib::update_music_stream".as_ptr(), Raylib_UpdateMusicStream);
    register_callback(collector_context, c"UpdateMusicStream".as_ptr(), Raylib_UpdateMusicStream);
    register_callback(collector_context, c"update_music_stream".as_ptr(), Raylib_UpdateMusicStream);
    register_callback(collector_context, c"Raylib::StopMusicStream".as_ptr(), Raylib_StopMusicStream);
    register_callback(collector_context, c"Raylib::stop_music_stream".as_ptr(), Raylib_StopMusicStream);
    register_callback(collector_context, c"raylib::StopMusicStream".as_ptr(), Raylib_StopMusicStream);
    register_callback(collector_context, c"raylib::stop_music_stream".as_ptr(), Raylib_StopMusicStream);
    register_callback(collector_context, c"StopMusicStream".as_ptr(), Raylib_StopMusicStream);
    register_callback(collector_context, c"stop_music_stream".as_ptr(), Raylib_StopMusicStream);
    register_callback(collector_context, c"Raylib::PauseMusicStream".as_ptr(), Raylib_PauseMusicStream);
    register_callback(collector_context, c"Raylib::pause_music_stream".as_ptr(), Raylib_PauseMusicStream);
    register_callback(collector_context, c"raylib::PauseMusicStream".as_ptr(), Raylib_PauseMusicStream);
    register_callback(collector_context, c"raylib::pause_music_stream".as_ptr(), Raylib_PauseMusicStream);
    register_callback(collector_context, c"PauseMusicStream".as_ptr(), Raylib_PauseMusicStream);
    register_callback(collector_context, c"pause_music_stream".as_ptr(), Raylib_PauseMusicStream);
    register_callback(collector_context, c"Raylib::ResumeMusicStream".as_ptr(), Raylib_ResumeMusicStream);
    register_callback(collector_context, c"Raylib::resume_music_stream".as_ptr(), Raylib_ResumeMusicStream);
    register_callback(collector_context, c"raylib::ResumeMusicStream".as_ptr(), Raylib_ResumeMusicStream);
    register_callback(collector_context, c"raylib::resume_music_stream".as_ptr(), Raylib_ResumeMusicStream);
    register_callback(collector_context, c"ResumeMusicStream".as_ptr(), Raylib_ResumeMusicStream);
    register_callback(collector_context, c"resume_music_stream".as_ptr(), Raylib_ResumeMusicStream);
    register_callback(collector_context, c"Raylib::SeekMusicStream".as_ptr(), Raylib_SeekMusicStream);
    register_callback(collector_context, c"Raylib::seek_music_stream".as_ptr(), Raylib_SeekMusicStream);
    register_callback(collector_context, c"raylib::SeekMusicStream".as_ptr(), Raylib_SeekMusicStream);
    register_callback(collector_context, c"raylib::seek_music_stream".as_ptr(), Raylib_SeekMusicStream);
    register_callback(collector_context, c"SeekMusicStream".as_ptr(), Raylib_SeekMusicStream);
    register_callback(collector_context, c"seek_music_stream".as_ptr(), Raylib_SeekMusicStream);
    register_callback(collector_context, c"Raylib::SetMusicVolume".as_ptr(), Raylib_SetMusicVolume);
    register_callback(collector_context, c"Raylib::set_music_volume".as_ptr(), Raylib_SetMusicVolume);
    register_callback(collector_context, c"raylib::SetMusicVolume".as_ptr(), Raylib_SetMusicVolume);
    register_callback(collector_context, c"raylib::set_music_volume".as_ptr(), Raylib_SetMusicVolume);
    register_callback(collector_context, c"SetMusicVolume".as_ptr(), Raylib_SetMusicVolume);
    register_callback(collector_context, c"set_music_volume".as_ptr(), Raylib_SetMusicVolume);
    register_callback(collector_context, c"Raylib::SetMusicPitch".as_ptr(), Raylib_SetMusicPitch);
    register_callback(collector_context, c"Raylib::set_music_pitch".as_ptr(), Raylib_SetMusicPitch);
    register_callback(collector_context, c"raylib::SetMusicPitch".as_ptr(), Raylib_SetMusicPitch);
    register_callback(collector_context, c"raylib::set_music_pitch".as_ptr(), Raylib_SetMusicPitch);
    register_callback(collector_context, c"SetMusicPitch".as_ptr(), Raylib_SetMusicPitch);
    register_callback(collector_context, c"set_music_pitch".as_ptr(), Raylib_SetMusicPitch);
    register_callback(collector_context, c"Raylib::SetMusicPan".as_ptr(), Raylib_SetMusicPan);
    register_callback(collector_context, c"Raylib::set_music_pan".as_ptr(), Raylib_SetMusicPan);
    register_callback(collector_context, c"raylib::SetMusicPan".as_ptr(), Raylib_SetMusicPan);
    register_callback(collector_context, c"raylib::set_music_pan".as_ptr(), Raylib_SetMusicPan);
    register_callback(collector_context, c"SetMusicPan".as_ptr(), Raylib_SetMusicPan);
    register_callback(collector_context, c"set_music_pan".as_ptr(), Raylib_SetMusicPan);
    register_callback(collector_context, c"Raylib::GetMusicTimeLength".as_ptr(), Raylib_GetMusicTimeLength);
    register_callback(collector_context, c"Raylib::get_music_time_length".as_ptr(), Raylib_GetMusicTimeLength);
    register_callback(collector_context, c"raylib::GetMusicTimeLength".as_ptr(), Raylib_GetMusicTimeLength);
    register_callback(collector_context, c"raylib::get_music_time_length".as_ptr(), Raylib_GetMusicTimeLength);
    register_callback(collector_context, c"GetMusicTimeLength".as_ptr(), Raylib_GetMusicTimeLength);
    register_callback(collector_context, c"get_music_time_length".as_ptr(), Raylib_GetMusicTimeLength);
    register_callback(collector_context, c"Raylib::GetMusicTimePlayed".as_ptr(), Raylib_GetMusicTimePlayed);
    register_callback(collector_context, c"Raylib::get_music_time_played".as_ptr(), Raylib_GetMusicTimePlayed);
    register_callback(collector_context, c"raylib::GetMusicTimePlayed".as_ptr(), Raylib_GetMusicTimePlayed);
    register_callback(collector_context, c"raylib::get_music_time_played".as_ptr(), Raylib_GetMusicTimePlayed);
    register_callback(collector_context, c"GetMusicTimePlayed".as_ptr(), Raylib_GetMusicTimePlayed);
    register_callback(collector_context, c"get_music_time_played".as_ptr(), Raylib_GetMusicTimePlayed);
    register_callback(collector_context, c"Raylib::LoadAudioStream".as_ptr(), Raylib_LoadAudioStream);
    register_callback(collector_context, c"Raylib::load_audio_stream".as_ptr(), Raylib_LoadAudioStream);
    register_callback(collector_context, c"raylib::LoadAudioStream".as_ptr(), Raylib_LoadAudioStream);
    register_callback(collector_context, c"raylib::load_audio_stream".as_ptr(), Raylib_LoadAudioStream);
    register_callback(collector_context, c"LoadAudioStream".as_ptr(), Raylib_LoadAudioStream);
    register_callback(collector_context, c"load_audio_stream".as_ptr(), Raylib_LoadAudioStream);
    register_callback(collector_context, c"Raylib::IsAudioStreamValid".as_ptr(), Raylib_IsAudioStreamValid);
    register_callback(collector_context, c"Raylib::is_audio_stream_valid".as_ptr(), Raylib_IsAudioStreamValid);
    register_callback(collector_context, c"raylib::IsAudioStreamValid".as_ptr(), Raylib_IsAudioStreamValid);
    register_callback(collector_context, c"raylib::is_audio_stream_valid".as_ptr(), Raylib_IsAudioStreamValid);
    register_callback(collector_context, c"IsAudioStreamValid".as_ptr(), Raylib_IsAudioStreamValid);
    register_callback(collector_context, c"is_audio_stream_valid".as_ptr(), Raylib_IsAudioStreamValid);
    register_callback(collector_context, c"Raylib::UnloadAudioStream".as_ptr(), Raylib_UnloadAudioStream);
    register_callback(collector_context, c"Raylib::unload_audio_stream".as_ptr(), Raylib_UnloadAudioStream);
    register_callback(collector_context, c"raylib::UnloadAudioStream".as_ptr(), Raylib_UnloadAudioStream);
    register_callback(collector_context, c"raylib::unload_audio_stream".as_ptr(), Raylib_UnloadAudioStream);
    register_callback(collector_context, c"UnloadAudioStream".as_ptr(), Raylib_UnloadAudioStream);
    register_callback(collector_context, c"unload_audio_stream".as_ptr(), Raylib_UnloadAudioStream);
    register_callback(collector_context, c"Raylib::UpdateAudioStream".as_ptr(), Raylib_UpdateAudioStream);
    register_callback(collector_context, c"Raylib::update_audio_stream".as_ptr(), Raylib_UpdateAudioStream);
    register_callback(collector_context, c"raylib::UpdateAudioStream".as_ptr(), Raylib_UpdateAudioStream);
    register_callback(collector_context, c"raylib::update_audio_stream".as_ptr(), Raylib_UpdateAudioStream);
    register_callback(collector_context, c"UpdateAudioStream".as_ptr(), Raylib_UpdateAudioStream);
    register_callback(collector_context, c"update_audio_stream".as_ptr(), Raylib_UpdateAudioStream);
    register_callback(collector_context, c"Raylib::IsAudioStreamProcessed".as_ptr(), Raylib_IsAudioStreamProcessed);
    register_callback(collector_context, c"Raylib::is_audio_stream_processed".as_ptr(), Raylib_IsAudioStreamProcessed);
    register_callback(collector_context, c"raylib::IsAudioStreamProcessed".as_ptr(), Raylib_IsAudioStreamProcessed);
    register_callback(collector_context, c"raylib::is_audio_stream_processed".as_ptr(), Raylib_IsAudioStreamProcessed);
    register_callback(collector_context, c"IsAudioStreamProcessed".as_ptr(), Raylib_IsAudioStreamProcessed);
    register_callback(collector_context, c"is_audio_stream_processed".as_ptr(), Raylib_IsAudioStreamProcessed);
    register_callback(collector_context, c"Raylib::PlayAudioStream".as_ptr(), Raylib_PlayAudioStream);
    register_callback(collector_context, c"Raylib::play_audio_stream".as_ptr(), Raylib_PlayAudioStream);
    register_callback(collector_context, c"raylib::PlayAudioStream".as_ptr(), Raylib_PlayAudioStream);
    register_callback(collector_context, c"raylib::play_audio_stream".as_ptr(), Raylib_PlayAudioStream);
    register_callback(collector_context, c"PlayAudioStream".as_ptr(), Raylib_PlayAudioStream);
    register_callback(collector_context, c"play_audio_stream".as_ptr(), Raylib_PlayAudioStream);
    register_callback(collector_context, c"Raylib::PauseAudioStream".as_ptr(), Raylib_PauseAudioStream);
    register_callback(collector_context, c"Raylib::pause_audio_stream".as_ptr(), Raylib_PauseAudioStream);
    register_callback(collector_context, c"raylib::PauseAudioStream".as_ptr(), Raylib_PauseAudioStream);
    register_callback(collector_context, c"raylib::pause_audio_stream".as_ptr(), Raylib_PauseAudioStream);
    register_callback(collector_context, c"PauseAudioStream".as_ptr(), Raylib_PauseAudioStream);
    register_callback(collector_context, c"pause_audio_stream".as_ptr(), Raylib_PauseAudioStream);
    register_callback(collector_context, c"Raylib::ResumeAudioStream".as_ptr(), Raylib_ResumeAudioStream);
    register_callback(collector_context, c"Raylib::resume_audio_stream".as_ptr(), Raylib_ResumeAudioStream);
    register_callback(collector_context, c"raylib::ResumeAudioStream".as_ptr(), Raylib_ResumeAudioStream);
    register_callback(collector_context, c"raylib::resume_audio_stream".as_ptr(), Raylib_ResumeAudioStream);
    register_callback(collector_context, c"ResumeAudioStream".as_ptr(), Raylib_ResumeAudioStream);
    register_callback(collector_context, c"resume_audio_stream".as_ptr(), Raylib_ResumeAudioStream);
    register_callback(collector_context, c"Raylib::IsAudioStreamPlaying".as_ptr(), Raylib_IsAudioStreamPlaying);
    register_callback(collector_context, c"Raylib::is_audio_stream_playing".as_ptr(), Raylib_IsAudioStreamPlaying);
    register_callback(collector_context, c"raylib::IsAudioStreamPlaying".as_ptr(), Raylib_IsAudioStreamPlaying);
    register_callback(collector_context, c"raylib::is_audio_stream_playing".as_ptr(), Raylib_IsAudioStreamPlaying);
    register_callback(collector_context, c"IsAudioStreamPlaying".as_ptr(), Raylib_IsAudioStreamPlaying);
    register_callback(collector_context, c"is_audio_stream_playing".as_ptr(), Raylib_IsAudioStreamPlaying);
    register_callback(collector_context, c"Raylib::StopAudioStream".as_ptr(), Raylib_StopAudioStream);
    register_callback(collector_context, c"Raylib::stop_audio_stream".as_ptr(), Raylib_StopAudioStream);
    register_callback(collector_context, c"raylib::StopAudioStream".as_ptr(), Raylib_StopAudioStream);
    register_callback(collector_context, c"raylib::stop_audio_stream".as_ptr(), Raylib_StopAudioStream);
    register_callback(collector_context, c"StopAudioStream".as_ptr(), Raylib_StopAudioStream);
    register_callback(collector_context, c"stop_audio_stream".as_ptr(), Raylib_StopAudioStream);
    register_callback(collector_context, c"Raylib::SetAudioStreamVolume".as_ptr(), Raylib_SetAudioStreamVolume);
    register_callback(collector_context, c"Raylib::set_audio_stream_volume".as_ptr(), Raylib_SetAudioStreamVolume);
    register_callback(collector_context, c"raylib::SetAudioStreamVolume".as_ptr(), Raylib_SetAudioStreamVolume);
    register_callback(collector_context, c"raylib::set_audio_stream_volume".as_ptr(), Raylib_SetAudioStreamVolume);
    register_callback(collector_context, c"SetAudioStreamVolume".as_ptr(), Raylib_SetAudioStreamVolume);
    register_callback(collector_context, c"set_audio_stream_volume".as_ptr(), Raylib_SetAudioStreamVolume);
    register_callback(collector_context, c"Raylib::SetAudioStreamPitch".as_ptr(), Raylib_SetAudioStreamPitch);
    register_callback(collector_context, c"Raylib::set_audio_stream_pitch".as_ptr(), Raylib_SetAudioStreamPitch);
    register_callback(collector_context, c"raylib::SetAudioStreamPitch".as_ptr(), Raylib_SetAudioStreamPitch);
    register_callback(collector_context, c"raylib::set_audio_stream_pitch".as_ptr(), Raylib_SetAudioStreamPitch);
    register_callback(collector_context, c"SetAudioStreamPitch".as_ptr(), Raylib_SetAudioStreamPitch);
    register_callback(collector_context, c"set_audio_stream_pitch".as_ptr(), Raylib_SetAudioStreamPitch);
    register_callback(collector_context, c"Raylib::SetAudioStreamPan".as_ptr(), Raylib_SetAudioStreamPan);
    register_callback(collector_context, c"Raylib::set_audio_stream_pan".as_ptr(), Raylib_SetAudioStreamPan);
    register_callback(collector_context, c"raylib::SetAudioStreamPan".as_ptr(), Raylib_SetAudioStreamPan);
    register_callback(collector_context, c"raylib::set_audio_stream_pan".as_ptr(), Raylib_SetAudioStreamPan);
    register_callback(collector_context, c"SetAudioStreamPan".as_ptr(), Raylib_SetAudioStreamPan);
    register_callback(collector_context, c"set_audio_stream_pan".as_ptr(), Raylib_SetAudioStreamPan);
    register_callback(collector_context, c"Raylib::SetAudioStreamBufferSizeDefault".as_ptr(), Raylib_SetAudioStreamBufferSizeDefault);
    register_callback(collector_context, c"Raylib::set_audio_stream_buffer_size_default".as_ptr(), Raylib_SetAudioStreamBufferSizeDefault);
    register_callback(collector_context, c"raylib::SetAudioStreamBufferSizeDefault".as_ptr(), Raylib_SetAudioStreamBufferSizeDefault);
    register_callback(collector_context, c"raylib::set_audio_stream_buffer_size_default".as_ptr(), Raylib_SetAudioStreamBufferSizeDefault);
    register_callback(collector_context, c"SetAudioStreamBufferSizeDefault".as_ptr(), Raylib_SetAudioStreamBufferSizeDefault);
    register_callback(collector_context, c"set_audio_stream_buffer_size_default".as_ptr(), Raylib_SetAudioStreamBufferSizeDefault);
    register_callback(collector_context, c"Raylib::SetAudioStreamCallback".as_ptr(), Raylib_SetAudioStreamCallback);
    register_callback(collector_context, c"Raylib::set_audio_stream_callback".as_ptr(), Raylib_SetAudioStreamCallback);
    register_callback(collector_context, c"raylib::SetAudioStreamCallback".as_ptr(), Raylib_SetAudioStreamCallback);
    register_callback(collector_context, c"raylib::set_audio_stream_callback".as_ptr(), Raylib_SetAudioStreamCallback);
    register_callback(collector_context, c"SetAudioStreamCallback".as_ptr(), Raylib_SetAudioStreamCallback);
    register_callback(collector_context, c"set_audio_stream_callback".as_ptr(), Raylib_SetAudioStreamCallback);
    register_callback(collector_context, c"Raylib::AttachAudioStreamProcessor".as_ptr(), Raylib_AttachAudioStreamProcessor);
    register_callback(collector_context, c"Raylib::attach_audio_stream_processor".as_ptr(), Raylib_AttachAudioStreamProcessor);
    register_callback(collector_context, c"raylib::AttachAudioStreamProcessor".as_ptr(), Raylib_AttachAudioStreamProcessor);
    register_callback(collector_context, c"raylib::attach_audio_stream_processor".as_ptr(), Raylib_AttachAudioStreamProcessor);
    register_callback(collector_context, c"AttachAudioStreamProcessor".as_ptr(), Raylib_AttachAudioStreamProcessor);
    register_callback(collector_context, c"attach_audio_stream_processor".as_ptr(), Raylib_AttachAudioStreamProcessor);
    register_callback(collector_context, c"Raylib::DetachAudioStreamProcessor".as_ptr(), Raylib_DetachAudioStreamProcessor);
    register_callback(collector_context, c"Raylib::detach_audio_stream_processor".as_ptr(), Raylib_DetachAudioStreamProcessor);
    register_callback(collector_context, c"raylib::DetachAudioStreamProcessor".as_ptr(), Raylib_DetachAudioStreamProcessor);
    register_callback(collector_context, c"raylib::detach_audio_stream_processor".as_ptr(), Raylib_DetachAudioStreamProcessor);
    register_callback(collector_context, c"DetachAudioStreamProcessor".as_ptr(), Raylib_DetachAudioStreamProcessor);
    register_callback(collector_context, c"detach_audio_stream_processor".as_ptr(), Raylib_DetachAudioStreamProcessor);
    register_callback(collector_context, c"Raylib::AttachAudioMixedProcessor".as_ptr(), Raylib_AttachAudioMixedProcessor);
    register_callback(collector_context, c"Raylib::attach_audio_mixed_processor".as_ptr(), Raylib_AttachAudioMixedProcessor);
    register_callback(collector_context, c"raylib::AttachAudioMixedProcessor".as_ptr(), Raylib_AttachAudioMixedProcessor);
    register_callback(collector_context, c"raylib::attach_audio_mixed_processor".as_ptr(), Raylib_AttachAudioMixedProcessor);
    register_callback(collector_context, c"AttachAudioMixedProcessor".as_ptr(), Raylib_AttachAudioMixedProcessor);
    register_callback(collector_context, c"attach_audio_mixed_processor".as_ptr(), Raylib_AttachAudioMixedProcessor);
    register_callback(collector_context, c"Raylib::DetachAudioMixedProcessor".as_ptr(), Raylib_DetachAudioMixedProcessor);
    register_callback(collector_context, c"Raylib::detach_audio_mixed_processor".as_ptr(), Raylib_DetachAudioMixedProcessor);
    register_callback(collector_context, c"raylib::DetachAudioMixedProcessor".as_ptr(), Raylib_DetachAudioMixedProcessor);
    register_callback(collector_context, c"raylib::detach_audio_mixed_processor".as_ptr(), Raylib_DetachAudioMixedProcessor);
    register_callback(collector_context, c"DetachAudioMixedProcessor".as_ptr(), Raylib_DetachAudioMixedProcessor);
    register_callback(collector_context, c"detach_audio_mixed_processor".as_ptr(), Raylib_DetachAudioMixedProcessor);
}

