//! Auto-generated 1-to-1 Rust native bridge extension for Solix C-API bindings.
//! Generated via unified Two-Pass IR generator using `solix-ffi`.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_variables, dead_code, unused_imports, unused_mut, clippy::all)]

use solix_ffi::{SolixCValue, SolixVmContext, solix_export};
use std::ffi::{
    CStr, CString, c_char, c_int, c_longlong, c_short, c_uint, c_ulonglong, c_ushort, c_void,
};

// ===========================================================================
// Callback Function Signatures
// ===========================================================================
pub type TraceLogCallback = Option<unsafe extern "C" fn(log_level: c_int, text: *const c_char, args: *mut c_void)>;
pub type LoadFileDataCallback = Option<unsafe extern "C" fn(file_name: *const c_char, data_size: *mut c_int) -> *mut u8>;
pub type SaveFileDataCallback = Option<unsafe extern "C" fn(file_name: *const c_char, data: *mut c_void, data_size: c_int) -> bool>;
pub type LoadFileTextCallback = Option<unsafe extern "C" fn(file_name: *const c_char) -> *const c_char>;
pub type SaveFileTextCallback = Option<unsafe extern "C" fn(file_name: *const c_char, text: *const c_char) -> bool>;
pub type AudioCallback = Option<unsafe extern "C" fn(buffer_data: *mut c_void, frames: c_uint)>;
pub type rlglLoadProc = Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>;

// ===========================================================================
// C ABI Struct Definitions & Aliases
// ===========================================================================
pub type C_Quaternion = C_Vector4;
pub type C_Texture2D = C_Texture;
pub type C_TextureCubemap = C_Texture;
pub type C_RenderTexture2D = C_RenderTexture;
pub type C_Camera = C_Camera3D;
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
    pub texture: C_Texture,
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
    pub texture: C_Texture,
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
    pub rotation: C_Vector4,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_rlVertexBuffer {
    pub elementCount: c_int,
    pub vertices: *mut f32,
    pub texcoords: *mut f32,
    pub normals: *mut f32,
    pub colors: *mut u8,
    pub indices: *mut c_uint,
    pub vaoId: c_uint,
    pub vboId: [c_uint; 5],
}

impl Default for C_rlVertexBuffer {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_rlDrawCall {
    pub mode: c_int,
    pub vertexCount: c_int,
    pub vertexAlignment: c_int,
    pub textureId: c_uint,
}

impl Default for C_rlDrawCall {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_rlRenderBatch {
    pub bufferCount: c_int,
    pub currentBuffer: c_int,
    pub vertexBuffer: *mut C_rlVertexBuffer,
    pub draws: *mut C_rlDrawCall,
    pub drawCounter: c_int,
    pub currentDepth: f32,
}

impl Default for C_rlRenderBatch {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_float3 {
    pub v: [f32; 3],
}

impl Default for C_float3 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct C_float16 {
    pub v: [f32; 16],
}

impl Default for C_float16 {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// ===========================================================================
// Struct Unpack, Pack & Writeback Helpers
// ===========================================================================
unsafe fn writeback_vector2(vm_context: *mut c_void, addr: u32, val: &C_Vector2) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.x.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 1, val.y.to_bits() as u64);
}

unsafe fn pack_vector2(vm_context: *mut c_void, val: C_Vector2) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Vector2");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_vector2(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vector2(vm_context: *mut c_void, addr: u32) -> C_Vector2 {
    if addr == 0 { return C_Vector2::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Vector2::default(); };
    let mut result = C_Vector2::default();
    result.x = f32::from_bits(ctx.read_field_u64_by_index(addr, 0) as u32);
    result.y = f32::from_bits(ctx.read_field_u64_by_index(addr, 1) as u32);
    result
}

unsafe fn writeback_vector3(vm_context: *mut c_void, addr: u32, val: &C_Vector3) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.x.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 1, val.y.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 2, val.z.to_bits() as u64);
}

unsafe fn pack_vector3(vm_context: *mut c_void, val: C_Vector3) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Vector3");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_vector3(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vector3(vm_context: *mut c_void, addr: u32) -> C_Vector3 {
    if addr == 0 { return C_Vector3::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Vector3::default(); };
    let mut result = C_Vector3::default();
    result.x = f32::from_bits(ctx.read_field_u64_by_index(addr, 0) as u32);
    result.y = f32::from_bits(ctx.read_field_u64_by_index(addr, 1) as u32);
    result.z = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result
}

unsafe fn writeback_vector4(vm_context: *mut c_void, addr: u32, val: &C_Vector4) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.x.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 1, val.y.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 2, val.z.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 3, val.w.to_bits() as u64);
}

unsafe fn pack_vector4(vm_context: *mut c_void, val: C_Vector4) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Vector4");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_vector4(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vector4(vm_context: *mut c_void, addr: u32) -> C_Vector4 {
    if addr == 0 { return C_Vector4::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Vector4::default(); };
    let mut result = C_Vector4::default();
    result.x = f32::from_bits(ctx.read_field_u64_by_index(addr, 0) as u32);
    result.y = f32::from_bits(ctx.read_field_u64_by_index(addr, 1) as u32);
    result.z = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result.w = f32::from_bits(ctx.read_field_u64_by_index(addr, 3) as u32);
    result
}

unsafe fn writeback_matrix(vm_context: *mut c_void, addr: u32, val: &C_Matrix) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.m0.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 1, val.m4.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 2, val.m8.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 3, val.m12.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 4, val.m1.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 5, val.m5.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 6, val.m9.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 7, val.m13.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 8, val.m2.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 9, val.m6.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 10, val.m10.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 11, val.m14.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 12, val.m3.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 13, val.m7.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 14, val.m11.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 15, val.m15.to_bits() as u64);
}

unsafe fn pack_matrix(vm_context: *mut c_void, val: C_Matrix) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Matrix");
    let addr = ctx.alloc_object(class_vtable_id, 16);
    if addr > 0 {
        writeback_matrix(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_matrix(vm_context: *mut c_void, addr: u32) -> C_Matrix {
    if addr == 0 { return C_Matrix::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Matrix::default(); };
    let mut result = C_Matrix::default();
    result.m0 = f32::from_bits(ctx.read_field_u64_by_index(addr, 0) as u32);
    result.m4 = f32::from_bits(ctx.read_field_u64_by_index(addr, 1) as u32);
    result.m8 = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result.m12 = f32::from_bits(ctx.read_field_u64_by_index(addr, 3) as u32);
    result.m1 = f32::from_bits(ctx.read_field_u64_by_index(addr, 4) as u32);
    result.m5 = f32::from_bits(ctx.read_field_u64_by_index(addr, 5) as u32);
    result.m9 = f32::from_bits(ctx.read_field_u64_by_index(addr, 6) as u32);
    result.m13 = f32::from_bits(ctx.read_field_u64_by_index(addr, 7) as u32);
    result.m2 = f32::from_bits(ctx.read_field_u64_by_index(addr, 8) as u32);
    result.m6 = f32::from_bits(ctx.read_field_u64_by_index(addr, 9) as u32);
    result.m10 = f32::from_bits(ctx.read_field_u64_by_index(addr, 10) as u32);
    result.m14 = f32::from_bits(ctx.read_field_u64_by_index(addr, 11) as u32);
    result.m3 = f32::from_bits(ctx.read_field_u64_by_index(addr, 12) as u32);
    result.m7 = f32::from_bits(ctx.read_field_u64_by_index(addr, 13) as u32);
    result.m11 = f32::from_bits(ctx.read_field_u64_by_index(addr, 14) as u32);
    result.m15 = f32::from_bits(ctx.read_field_u64_by_index(addr, 15) as u32);
    result
}

unsafe fn writeback_color(vm_context: *mut c_void, addr: u32, val: &C_Color) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.r as u64);
    ctx.write_field_u64_by_index(addr, 1, val.g as u64);
    ctx.write_field_u64_by_index(addr, 2, val.b as u64);
    ctx.write_field_u64_by_index(addr, 3, val.a as u64);
}

unsafe fn pack_color(vm_context: *mut c_void, val: C_Color) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Color");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_color(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_color(vm_context: *mut c_void, addr: u32) -> C_Color {
    if addr == 0 { return C_Color::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Color::default(); };
    let mut result = C_Color::default();
    result.r = ctx.read_field_u64_by_index(addr, 0) as u8;
    result.g = ctx.read_field_u64_by_index(addr, 1) as u8;
    result.b = ctx.read_field_u64_by_index(addr, 2) as u8;
    result.a = ctx.read_field_u64_by_index(addr, 3) as u8;
    result
}

unsafe fn writeback_rectangle(vm_context: *mut c_void, addr: u32, val: &C_Rectangle) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.x.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 1, val.y.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 2, val.width.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 3, val.height.to_bits() as u64);
}

unsafe fn pack_rectangle(vm_context: *mut c_void, val: C_Rectangle) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Rectangle");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_rectangle(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_rectangle(vm_context: *mut c_void, addr: u32) -> C_Rectangle {
    if addr == 0 { return C_Rectangle::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Rectangle::default(); };
    let mut result = C_Rectangle::default();
    result.x = f32::from_bits(ctx.read_field_u64_by_index(addr, 0) as u32);
    result.y = f32::from_bits(ctx.read_field_u64_by_index(addr, 1) as u32);
    result.width = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result.height = f32::from_bits(ctx.read_field_u64_by_index(addr, 3) as u32);
    result
}

unsafe fn writeback_image(vm_context: *mut c_void, addr: u32, val: &C_Image) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.data as usize as u64);
    ctx.set_field_i32_by_index(addr, 1, val.width as i32);
    ctx.set_field_i32_by_index(addr, 2, val.height as i32);
    ctx.set_field_i32_by_index(addr, 3, val.mipmaps as i32);
    ctx.set_field_i32_by_index(addr, 4, val.format as i32);
}

unsafe fn pack_image(vm_context: *mut c_void, val: C_Image) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Image");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_image(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_image(vm_context: *mut c_void, addr: u32) -> C_Image {
    if addr == 0 { return C_Image::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Image::default(); };
    let mut result = C_Image::default();
    result.data = ctx.read_field_u64_by_index(addr, 0) as usize as *mut c_void;
    result.width = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.height = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.mipmaps = ctx.get_field_i32_by_index(addr, 3) as c_int;
    result.format = ctx.get_field_i32_by_index(addr, 4) as c_int;
    result
}

unsafe fn writeback_texture(vm_context: *mut c_void, addr: u32, val: &C_Texture) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.id as u64);
    ctx.set_field_i32_by_index(addr, 1, val.width as i32);
    ctx.set_field_i32_by_index(addr, 2, val.height as i32);
    ctx.set_field_i32_by_index(addr, 3, val.mipmaps as i32);
    ctx.set_field_i32_by_index(addr, 4, val.format as i32);
}

unsafe fn pack_texture(vm_context: *mut c_void, val: C_Texture) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Texture");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_texture(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_texture(vm_context: *mut c_void, addr: u32) -> C_Texture {
    if addr == 0 { return C_Texture::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Texture::default(); };
    let mut result = C_Texture::default();
    result.id = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.width = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.height = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.mipmaps = ctx.get_field_i32_by_index(addr, 3) as c_int;
    result.format = ctx.get_field_i32_by_index(addr, 4) as c_int;
    result
}

unsafe fn writeback_render_texture(vm_context: *mut c_void, addr: u32, val: &C_RenderTexture) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.id as u64);
    let child_1 = pack_texture(vm_context, val.texture);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
    let child_2 = pack_texture(vm_context, val.depth);
    ctx.write_field_u64_by_index(addr, 2, child_2 as u64);
}

unsafe fn pack_render_texture(vm_context: *mut c_void, val: C_RenderTexture) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("RenderTexture");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_render_texture(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_render_texture(vm_context: *mut c_void, addr: u32) -> C_RenderTexture {
    if addr == 0 { return C_RenderTexture::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_RenderTexture::default(); };
    let mut result = C_RenderTexture::default();
    result.id = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.texture = unpack_texture(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result.depth = unpack_texture(vm_context, ctx.read_field_u64_by_index(addr, 2) as u32);
    result
}

unsafe fn writeback_n_patch_info(vm_context: *mut c_void, addr: u32, val: &C_NPatchInfo) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_rectangle(vm_context, val.source);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    ctx.set_field_i32_by_index(addr, 1, val.left as i32);
    ctx.set_field_i32_by_index(addr, 2, val.top as i32);
    ctx.set_field_i32_by_index(addr, 3, val.right as i32);
    ctx.set_field_i32_by_index(addr, 4, val.bottom as i32);
    ctx.set_field_i32_by_index(addr, 5, val.layout as i32);
}

unsafe fn pack_n_patch_info(vm_context: *mut c_void, val: C_NPatchInfo) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("NPatchInfo");
    let addr = ctx.alloc_object(class_vtable_id, 6);
    if addr > 0 {
        writeback_n_patch_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_n_patch_info(vm_context: *mut c_void, addr: u32) -> C_NPatchInfo {
    if addr == 0 { return C_NPatchInfo::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_NPatchInfo::default(); };
    let mut result = C_NPatchInfo::default();
    result.source = unpack_rectangle(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.left = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.top = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.right = ctx.get_field_i32_by_index(addr, 3) as c_int;
    result.bottom = ctx.get_field_i32_by_index(addr, 4) as c_int;
    result.layout = ctx.get_field_i32_by_index(addr, 5) as c_int;
    result
}

unsafe fn writeback_glyph_info(vm_context: *mut c_void, addr: u32, val: &C_GlyphInfo) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.value as i32);
    ctx.set_field_i32_by_index(addr, 1, val.offsetX as i32);
    ctx.set_field_i32_by_index(addr, 2, val.offsetY as i32);
    ctx.set_field_i32_by_index(addr, 3, val.advanceX as i32);
    let child_4 = pack_image(vm_context, val.image);
    ctx.write_field_u64_by_index(addr, 4, child_4 as u64);
}

unsafe fn pack_glyph_info(vm_context: *mut c_void, val: C_GlyphInfo) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("GlyphInfo");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_glyph_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_glyph_info(vm_context: *mut c_void, addr: u32) -> C_GlyphInfo {
    if addr == 0 { return C_GlyphInfo::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_GlyphInfo::default(); };
    let mut result = C_GlyphInfo::default();
    result.value = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.offsetX = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.offsetY = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.advanceX = ctx.get_field_i32_by_index(addr, 3) as c_int;
    result.image = unpack_image(vm_context, ctx.read_field_u64_by_index(addr, 4) as u32);
    result
}

unsafe fn writeback_font(vm_context: *mut c_void, addr: u32, val: &C_Font) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.baseSize as i32);
    ctx.set_field_i32_by_index(addr, 1, val.glyphCount as i32);
    ctx.set_field_i32_by_index(addr, 2, val.glyphPadding as i32);
    let child_3 = pack_texture(vm_context, val.texture);
    ctx.write_field_u64_by_index(addr, 3, child_3 as u64);
    ctx.write_field_u64_by_index(addr, 4, val.recs as usize as u64);
    ctx.write_field_u64_by_index(addr, 5, val.glyphs as usize as u64);
}

unsafe fn pack_font(vm_context: *mut c_void, val: C_Font) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Font");
    let addr = ctx.alloc_object(class_vtable_id, 6);
    if addr > 0 {
        writeback_font(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_font(vm_context: *mut c_void, addr: u32) -> C_Font {
    if addr == 0 { return C_Font::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Font::default(); };
    let mut result = C_Font::default();
    result.baseSize = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.glyphCount = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.glyphPadding = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.texture = unpack_texture(vm_context, ctx.read_field_u64_by_index(addr, 3) as u32);
    result.recs = ctx.read_field_u64_by_index(addr, 4) as usize as *mut C_Rectangle;
    result.glyphs = ctx.read_field_u64_by_index(addr, 5) as usize as *mut C_GlyphInfo;
    result
}

unsafe fn writeback_camera3d(vm_context: *mut c_void, addr: u32, val: &C_Camera3D) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_vector3(vm_context, val.position);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    let child_1 = pack_vector3(vm_context, val.target);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
    let child_2 = pack_vector3(vm_context, val.up);
    ctx.write_field_u64_by_index(addr, 2, child_2 as u64);
    ctx.write_field_u64_by_index(addr, 3, val.fovy.to_bits() as u64);
    ctx.set_field_i32_by_index(addr, 4, val.projection as i32);
}

unsafe fn pack_camera3d(vm_context: *mut c_void, val: C_Camera3D) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Camera3D");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_camera3d(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_camera3d(vm_context: *mut c_void, addr: u32) -> C_Camera3D {
    if addr == 0 { return C_Camera3D::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Camera3D::default(); };
    let mut result = C_Camera3D::default();
    result.position = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.target = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result.up = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 2) as u32);
    result.fovy = f32::from_bits(ctx.read_field_u64_by_index(addr, 3) as u32);
    result.projection = ctx.get_field_i32_by_index(addr, 4) as c_int;
    result
}

unsafe fn writeback_camera2d(vm_context: *mut c_void, addr: u32, val: &C_Camera2D) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_vector2(vm_context, val.offset);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    let child_1 = pack_vector2(vm_context, val.target);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
    ctx.write_field_u64_by_index(addr, 2, val.rotation.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 3, val.zoom.to_bits() as u64);
}

unsafe fn pack_camera2d(vm_context: *mut c_void, val: C_Camera2D) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Camera2D");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_camera2d(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_camera2d(vm_context: *mut c_void, addr: u32) -> C_Camera2D {
    if addr == 0 { return C_Camera2D::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Camera2D::default(); };
    let mut result = C_Camera2D::default();
    result.offset = unpack_vector2(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.target = unpack_vector2(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result.rotation = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result.zoom = f32::from_bits(ctx.read_field_u64_by_index(addr, 3) as u32);
    result
}

unsafe fn writeback_mesh(vm_context: *mut c_void, addr: u32, val: &C_Mesh) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.vertexCount as i32);
    ctx.set_field_i32_by_index(addr, 1, val.triangleCount as i32);
    ctx.write_field_u64_by_index(addr, 2, val.vertices as usize as u64);
    ctx.write_field_u64_by_index(addr, 3, val.texcoords as usize as u64);
    ctx.write_field_u64_by_index(addr, 4, val.texcoords2 as usize as u64);
    ctx.write_field_u64_by_index(addr, 5, val.normals as usize as u64);
    ctx.write_field_u64_by_index(addr, 6, val.tangents as usize as u64);
    ctx.write_field_u64_by_index(addr, 7, val.colors as usize as u64);
    ctx.write_field_u64_by_index(addr, 8, val.indices as usize as u64);
    ctx.set_field_i32_by_index(addr, 9, val.boneCount as i32);
    ctx.write_field_u64_by_index(addr, 10, val.boneIndices as usize as u64);
    ctx.write_field_u64_by_index(addr, 11, val.boneWeights as usize as u64);
    ctx.write_field_u64_by_index(addr, 12, val.animVertices as usize as u64);
    ctx.write_field_u64_by_index(addr, 13, val.animNormals as usize as u64);
    ctx.write_field_u64_by_index(addr, 14, val.vaoId as u64);
    ctx.write_field_u64_by_index(addr, 15, val.vboId as usize as u64);
}

unsafe fn pack_mesh(vm_context: *mut c_void, val: C_Mesh) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Mesh");
    let addr = ctx.alloc_object(class_vtable_id, 16);
    if addr > 0 {
        writeback_mesh(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_mesh(vm_context: *mut c_void, addr: u32) -> C_Mesh {
    if addr == 0 { return C_Mesh::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Mesh::default(); };
    let mut result = C_Mesh::default();
    result.vertexCount = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.triangleCount = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.vertices = ctx.read_field_u64_by_index(addr, 2) as usize as *mut f32;
    result.texcoords = ctx.read_field_u64_by_index(addr, 3) as usize as *mut f32;
    result.texcoords2 = ctx.read_field_u64_by_index(addr, 4) as usize as *mut f32;
    result.normals = ctx.read_field_u64_by_index(addr, 5) as usize as *mut f32;
    result.tangents = ctx.read_field_u64_by_index(addr, 6) as usize as *mut f32;
    result.colors = ctx.read_field_u64_by_index(addr, 7) as usize as *mut u8;
    result.indices = ctx.read_field_u64_by_index(addr, 8) as usize as *mut c_ushort;
    result.boneCount = ctx.get_field_i32_by_index(addr, 9) as c_int;
    result.boneIndices = ctx.read_field_u64_by_index(addr, 10) as usize as *mut u8;
    result.boneWeights = ctx.read_field_u64_by_index(addr, 11) as usize as *mut f32;
    result.animVertices = ctx.read_field_u64_by_index(addr, 12) as usize as *mut f32;
    result.animNormals = ctx.read_field_u64_by_index(addr, 13) as usize as *mut f32;
    result.vaoId = ctx.read_field_u64_by_index(addr, 14) as c_uint;
    result.vboId = ctx.read_field_u64_by_index(addr, 15) as usize as *mut c_uint;
    result
}

unsafe fn writeback_shader(vm_context: *mut c_void, addr: u32, val: &C_Shader) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.id as u64);
    ctx.write_field_u64_by_index(addr, 1, val.locs as usize as u64);
}

unsafe fn pack_shader(vm_context: *mut c_void, val: C_Shader) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Shader");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_shader(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_shader(vm_context: *mut c_void, addr: u32) -> C_Shader {
    if addr == 0 { return C_Shader::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Shader::default(); };
    let mut result = C_Shader::default();
    result.id = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.locs = ctx.read_field_u64_by_index(addr, 1) as usize as *mut c_int;
    result
}

unsafe fn writeback_material_map(vm_context: *mut c_void, addr: u32, val: &C_MaterialMap) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_texture(vm_context, val.texture);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    let child_1 = pack_color(vm_context, val.color);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
    ctx.write_field_u64_by_index(addr, 2, val.value.to_bits() as u64);
}

unsafe fn pack_material_map(vm_context: *mut c_void, val: C_MaterialMap) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("MaterialMap");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_material_map(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_material_map(vm_context: *mut c_void, addr: u32) -> C_MaterialMap {
    if addr == 0 { return C_MaterialMap::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_MaterialMap::default(); };
    let mut result = C_MaterialMap::default();
    result.texture = unpack_texture(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.color = unpack_color(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result.value = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result
}

unsafe fn writeback_material(vm_context: *mut c_void, addr: u32, val: &C_Material) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_shader(vm_context, val.shader);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    ctx.write_field_u64_by_index(addr, 1, val.maps as usize as u64);
    let arr_addr_2 = ctx.alloc_array(4, 0);
    if arr_addr_2 != 0 {
        for elem_idx in 0..4u32 {
            ctx.array_set_u64(arr_addr_2, elem_idx, (val.params[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 2, arr_addr_2 as u64);
    }
}

unsafe fn pack_material(vm_context: *mut c_void, val: C_Material) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Material");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_material(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_material(vm_context: *mut c_void, addr: u32) -> C_Material {
    if addr == 0 { return C_Material::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Material::default(); };
    let mut result = C_Material::default();
    result.shader = unpack_shader(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.maps = ctx.read_field_u64_by_index(addr, 1) as usize as *mut C_MaterialMap;
    let arr_addr_2 = ctx.read_field_u64_by_index(addr, 2) as u32;
    if arr_addr_2 > 0 {
        let arr_len = ctx.array_len(arr_addr_2).min(4);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
            result.params[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    result
}

unsafe fn writeback_transform(vm_context: *mut c_void, addr: u32, val: &C_Transform) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_vector3(vm_context, val.translation);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    let child_1 = pack_vector4(vm_context, val.rotation);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
    let child_2 = pack_vector3(vm_context, val.scale);
    ctx.write_field_u64_by_index(addr, 2, child_2 as u64);
}

unsafe fn pack_transform(vm_context: *mut c_void, val: C_Transform) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Transform");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_transform(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_transform(vm_context: *mut c_void, addr: u32) -> C_Transform {
    if addr == 0 { return C_Transform::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Transform::default(); };
    let mut result = C_Transform::default();
    result.translation = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.rotation = unpack_vector4(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result.scale = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 2) as u32);
    result
}

unsafe fn writeback_bone_info(vm_context: *mut c_void, addr: u32, val: &C_BoneInfo) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let mut str_len_0 = 0usize;
    while str_len_0 < 32 && val.name[str_len_0] != 0 {
        str_len_0 += 1;
    }
    let raw_slice_0 = std::slice::from_raw_parts(val.name.as_ptr().cast::<u8>(), str_len_0);
    let sanitized_0 = String::from_utf8_lossy(raw_slice_0);
    let str_addr_0 = ctx.alloc_string(sanitized_0.as_ref());
    if str_addr_0 != 0 {
        ctx.write_field_u64_by_index(addr, 0, str_addr_0 as u64);
    }
    ctx.set_field_i32_by_index(addr, 1, val.parent as i32);
}

unsafe fn pack_bone_info(vm_context: *mut c_void, val: C_BoneInfo) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("BoneInfo");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_bone_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_bone_info(vm_context: *mut c_void, addr: u32) -> C_BoneInfo {
    if addr == 0 { return C_BoneInfo::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_BoneInfo::default(); };
    let mut result = C_BoneInfo::default();
    let str_addr_0 = ctx.read_field_u64_by_index(addr, 0) as u32;
    if str_addr_0 > 0 && 32 > 0 {
        let max_copy = (32usize).saturating_sub(1);
        let out_slice = std::slice::from_raw_parts_mut(result.name.as_mut_ptr().cast::<u8>(), max_copy);
        let read_len = ctx.read_bytes(str_addr_0, out_slice).min(max_copy);
        result.name[read_len] = 0;
    }
    result.parent = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result
}

unsafe fn writeback_model_skeleton(vm_context: *mut c_void, addr: u32, val: &C_ModelSkeleton) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.boneCount as u64);
    ctx.write_field_u64_by_index(addr, 1, val.bones as usize as u64);
    ctx.write_field_u64_by_index(addr, 2, val.bindPose as usize as u64);
}

unsafe fn pack_model_skeleton(vm_context: *mut c_void, val: C_ModelSkeleton) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("ModelSkeleton");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_model_skeleton(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_model_skeleton(vm_context: *mut c_void, addr: u32) -> C_ModelSkeleton {
    if addr == 0 { return C_ModelSkeleton::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_ModelSkeleton::default(); };
    let mut result = C_ModelSkeleton::default();
    result.boneCount = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.bones = ctx.read_field_u64_by_index(addr, 1) as usize as *mut C_BoneInfo;
    result.bindPose = ctx.read_field_u64_by_index(addr, 2) as usize as *mut C_Transform;
    result
}

unsafe fn writeback_model(vm_context: *mut c_void, addr: u32, val: &C_Model) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_matrix(vm_context, val.transform);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    ctx.set_field_i32_by_index(addr, 1, val.meshCount as i32);
    ctx.set_field_i32_by_index(addr, 2, val.materialCount as i32);
    ctx.write_field_u64_by_index(addr, 3, val.meshes as usize as u64);
    ctx.write_field_u64_by_index(addr, 4, val.materials as usize as u64);
    ctx.write_field_u64_by_index(addr, 5, val.meshMaterial as usize as u64);
    let child_6 = pack_model_skeleton(vm_context, val.skeleton);
    ctx.write_field_u64_by_index(addr, 6, child_6 as u64);
    ctx.write_field_u64_by_index(addr, 7, val.currentPose as usize as u64);
    ctx.write_field_u64_by_index(addr, 8, val.boneMatrices as usize as u64);
}

unsafe fn pack_model(vm_context: *mut c_void, val: C_Model) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Model");
    let addr = ctx.alloc_object(class_vtable_id, 9);
    if addr > 0 {
        writeback_model(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_model(vm_context: *mut c_void, addr: u32) -> C_Model {
    if addr == 0 { return C_Model::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Model::default(); };
    let mut result = C_Model::default();
    result.transform = unpack_matrix(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.meshCount = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.materialCount = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.meshes = ctx.read_field_u64_by_index(addr, 3) as usize as *mut C_Mesh;
    result.materials = ctx.read_field_u64_by_index(addr, 4) as usize as *mut C_Material;
    result.meshMaterial = ctx.read_field_u64_by_index(addr, 5) as usize as *mut c_int;
    result.skeleton = unpack_model_skeleton(vm_context, ctx.read_field_u64_by_index(addr, 6) as u32);
    result.currentPose = ctx.read_field_u64_by_index(addr, 7) as usize as *mut C_Transform;
    result.boneMatrices = ctx.read_field_u64_by_index(addr, 8) as usize as *mut C_Matrix;
    result
}

unsafe fn writeback_model_animation(vm_context: *mut c_void, addr: u32, val: &C_ModelAnimation) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let mut str_len_0 = 0usize;
    while str_len_0 < 32 && val.name[str_len_0] != 0 {
        str_len_0 += 1;
    }
    let raw_slice_0 = std::slice::from_raw_parts(val.name.as_ptr().cast::<u8>(), str_len_0);
    let sanitized_0 = String::from_utf8_lossy(raw_slice_0);
    let str_addr_0 = ctx.alloc_string(sanitized_0.as_ref());
    if str_addr_0 != 0 {
        ctx.write_field_u64_by_index(addr, 0, str_addr_0 as u64);
    }
    ctx.write_field_u64_by_index(addr, 1, val.boneCount as u64);
    ctx.set_field_i32_by_index(addr, 2, val.keyframeCount as i32);
    ctx.write_field_u64_by_index(addr, 3, val.keyframePoses as usize as u64);
}

unsafe fn pack_model_animation(vm_context: *mut c_void, val: C_ModelAnimation) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("ModelAnimation");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_model_animation(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_model_animation(vm_context: *mut c_void, addr: u32) -> C_ModelAnimation {
    if addr == 0 { return C_ModelAnimation::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_ModelAnimation::default(); };
    let mut result = C_ModelAnimation::default();
    let str_addr_0 = ctx.read_field_u64_by_index(addr, 0) as u32;
    if str_addr_0 > 0 && 32 > 0 {
        let max_copy = (32usize).saturating_sub(1);
        let out_slice = std::slice::from_raw_parts_mut(result.name.as_mut_ptr().cast::<u8>(), max_copy);
        let read_len = ctx.read_bytes(str_addr_0, out_slice).min(max_copy);
        result.name[read_len] = 0;
    }
    result.boneCount = ctx.read_field_u64_by_index(addr, 1) as c_uint;
    result.keyframeCount = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.keyframePoses = ctx.read_field_u64_by_index(addr, 3) as usize as *mut *mut C_Transform;
    result
}

unsafe fn writeback_ray(vm_context: *mut c_void, addr: u32, val: &C_Ray) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_vector3(vm_context, val.position);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    let child_1 = pack_vector3(vm_context, val.direction);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
}

unsafe fn pack_ray(vm_context: *mut c_void, val: C_Ray) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Ray");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_ray(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_ray(vm_context: *mut c_void, addr: u32) -> C_Ray {
    if addr == 0 { return C_Ray::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Ray::default(); };
    let mut result = C_Ray::default();
    result.position = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.direction = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result
}

unsafe fn writeback_ray_collision(vm_context: *mut c_void, addr: u32, val: &C_RayCollision) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, if val.hit { 1 } else { 0 });
    ctx.write_field_u64_by_index(addr, 1, val.distance.to_bits() as u64);
    let child_2 = pack_vector3(vm_context, val.point);
    ctx.write_field_u64_by_index(addr, 2, child_2 as u64);
    let child_3 = pack_vector3(vm_context, val.normal);
    ctx.write_field_u64_by_index(addr, 3, child_3 as u64);
}

unsafe fn pack_ray_collision(vm_context: *mut c_void, val: C_RayCollision) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("RayCollision");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_ray_collision(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_ray_collision(vm_context: *mut c_void, addr: u32) -> C_RayCollision {
    if addr == 0 { return C_RayCollision::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_RayCollision::default(); };
    let mut result = C_RayCollision::default();
    result.hit = ctx.get_field_i32_by_index(addr, 0) != 0;
    result.distance = f32::from_bits(ctx.read_field_u64_by_index(addr, 1) as u32);
    result.point = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 2) as u32);
    result.normal = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 3) as u32);
    result
}

unsafe fn writeback_bounding_box(vm_context: *mut c_void, addr: u32, val: &C_BoundingBox) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_vector3(vm_context, val.min);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    let child_1 = pack_vector3(vm_context, val.max);
    ctx.write_field_u64_by_index(addr, 1, child_1 as u64);
}

unsafe fn pack_bounding_box(vm_context: *mut c_void, val: C_BoundingBox) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("BoundingBox");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_bounding_box(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_bounding_box(vm_context: *mut c_void, addr: u32) -> C_BoundingBox {
    if addr == 0 { return C_BoundingBox::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_BoundingBox::default(); };
    let mut result = C_BoundingBox::default();
    result.min = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.max = unpack_vector3(vm_context, ctx.read_field_u64_by_index(addr, 1) as u32);
    result
}

unsafe fn writeback_wave(vm_context: *mut c_void, addr: u32, val: &C_Wave) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.frameCount as u64);
    ctx.write_field_u64_by_index(addr, 1, val.sampleRate as u64);
    ctx.write_field_u64_by_index(addr, 2, val.sampleSize as u64);
    ctx.write_field_u64_by_index(addr, 3, val.channels as u64);
    ctx.write_field_u64_by_index(addr, 4, val.data as usize as u64);
}

unsafe fn pack_wave(vm_context: *mut c_void, val: C_Wave) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Wave");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_wave(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_wave(vm_context: *mut c_void, addr: u32) -> C_Wave {
    if addr == 0 { return C_Wave::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Wave::default(); };
    let mut result = C_Wave::default();
    result.frameCount = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.sampleRate = ctx.read_field_u64_by_index(addr, 1) as c_uint;
    result.sampleSize = ctx.read_field_u64_by_index(addr, 2) as c_uint;
    result.channels = ctx.read_field_u64_by_index(addr, 3) as c_uint;
    result.data = ctx.read_field_u64_by_index(addr, 4) as usize as *mut c_void;
    result
}

unsafe fn writeback_audio_stream(vm_context: *mut c_void, addr: u32, val: &C_AudioStream) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.buffer as usize as u64);
    ctx.write_field_u64_by_index(addr, 1, val.processor as usize as u64);
    ctx.write_field_u64_by_index(addr, 2, val.sampleRate as u64);
    ctx.write_field_u64_by_index(addr, 3, val.sampleSize as u64);
    ctx.write_field_u64_by_index(addr, 4, val.channels as u64);
}

unsafe fn pack_audio_stream(vm_context: *mut c_void, val: C_AudioStream) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("AudioStream");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_audio_stream(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_audio_stream(vm_context: *mut c_void, addr: u32) -> C_AudioStream {
    if addr == 0 { return C_AudioStream::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_AudioStream::default(); };
    let mut result = C_AudioStream::default();
    result.buffer = ctx.read_field_u64_by_index(addr, 0) as usize as *mut c_void;
    result.processor = ctx.read_field_u64_by_index(addr, 1) as usize as *mut c_void;
    result.sampleRate = ctx.read_field_u64_by_index(addr, 2) as c_uint;
    result.sampleSize = ctx.read_field_u64_by_index(addr, 3) as c_uint;
    result.channels = ctx.read_field_u64_by_index(addr, 4) as c_uint;
    result
}

unsafe fn writeback_sound(vm_context: *mut c_void, addr: u32, val: &C_Sound) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_audio_stream(vm_context, val.stream);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    ctx.write_field_u64_by_index(addr, 1, val.frameCount as u64);
}

unsafe fn pack_sound(vm_context: *mut c_void, val: C_Sound) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Sound");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_sound(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_sound(vm_context: *mut c_void, addr: u32) -> C_Sound {
    if addr == 0 { return C_Sound::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Sound::default(); };
    let mut result = C_Sound::default();
    result.stream = unpack_audio_stream(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.frameCount = ctx.read_field_u64_by_index(addr, 1) as c_uint;
    result
}

unsafe fn writeback_music(vm_context: *mut c_void, addr: u32, val: &C_Music) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let child_0 = pack_audio_stream(vm_context, val.stream);
    ctx.write_field_u64_by_index(addr, 0, child_0 as u64);
    ctx.write_field_u64_by_index(addr, 1, val.frameCount as u64);
    ctx.set_field_i32_by_index(addr, 2, if val.looping { 1 } else { 0 });
    ctx.set_field_i32_by_index(addr, 3, val.ctxType as i32);
    ctx.write_field_u64_by_index(addr, 4, val.ctxData as usize as u64);
}

unsafe fn pack_music(vm_context: *mut c_void, val: C_Music) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("Music");
    let addr = ctx.alloc_object(class_vtable_id, 5);
    if addr > 0 {
        writeback_music(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_music(vm_context: *mut c_void, addr: u32) -> C_Music {
    if addr == 0 { return C_Music::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_Music::default(); };
    let mut result = C_Music::default();
    result.stream = unpack_audio_stream(vm_context, ctx.read_field_u64_by_index(addr, 0) as u32);
    result.frameCount = ctx.read_field_u64_by_index(addr, 1) as c_uint;
    result.looping = ctx.get_field_i32_by_index(addr, 2) != 0;
    result.ctxType = ctx.get_field_i32_by_index(addr, 3) as c_int;
    result.ctxData = ctx.read_field_u64_by_index(addr, 4) as usize as *mut c_void;
    result
}

unsafe fn writeback_vr_device_info(vm_context: *mut c_void, addr: u32, val: &C_VrDeviceInfo) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.hResolution as i32);
    ctx.set_field_i32_by_index(addr, 1, val.vResolution as i32);
    ctx.write_field_u64_by_index(addr, 2, val.hScreenSize.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 3, val.vScreenSize.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 4, val.eyeToScreenDistance.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 5, val.lensSeparationDistance.to_bits() as u64);
    ctx.write_field_u64_by_index(addr, 6, val.interpupillaryDistance.to_bits() as u64);
    let arr_addr_7 = ctx.alloc_array(4, 0);
    if arr_addr_7 != 0 {
        for elem_idx in 0..4u32 {
            ctx.array_set_u64(arr_addr_7, elem_idx, (val.lensDistortionValues[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 7, arr_addr_7 as u64);
    }
    let arr_addr_8 = ctx.alloc_array(4, 0);
    if arr_addr_8 != 0 {
        for elem_idx in 0..4u32 {
            ctx.array_set_u64(arr_addr_8, elem_idx, (val.chromaAbCorrection[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 8, arr_addr_8 as u64);
    }
}

unsafe fn pack_vr_device_info(vm_context: *mut c_void, val: C_VrDeviceInfo) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("VrDeviceInfo");
    let addr = ctx.alloc_object(class_vtable_id, 9);
    if addr > 0 {
        writeback_vr_device_info(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vr_device_info(vm_context: *mut c_void, addr: u32) -> C_VrDeviceInfo {
    if addr == 0 { return C_VrDeviceInfo::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_VrDeviceInfo::default(); };
    let mut result = C_VrDeviceInfo::default();
    result.hResolution = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.vResolution = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.hScreenSize = f32::from_bits(ctx.read_field_u64_by_index(addr, 2) as u32);
    result.vScreenSize = f32::from_bits(ctx.read_field_u64_by_index(addr, 3) as u32);
    result.eyeToScreenDistance = f32::from_bits(ctx.read_field_u64_by_index(addr, 4) as u32);
    result.lensSeparationDistance = f32::from_bits(ctx.read_field_u64_by_index(addr, 5) as u32);
    result.interpupillaryDistance = f32::from_bits(ctx.read_field_u64_by_index(addr, 6) as u32);
    let arr_addr_7 = ctx.read_field_u64_by_index(addr, 7) as u32;
    if arr_addr_7 > 0 {
        let arr_len = ctx.array_len(arr_addr_7).min(4);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_7, elem_idx);
            result.lensDistortionValues[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    let arr_addr_8 = ctx.read_field_u64_by_index(addr, 8) as u32;
    if arr_addr_8 > 0 {
        let arr_len = ctx.array_len(arr_addr_8).min(4);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_8, elem_idx);
            result.chromaAbCorrection[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    result
}

unsafe fn writeback_vr_stereo_config(vm_context: *mut c_void, addr: u32, val: &C_VrStereoConfig) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let arr_addr_0 = ctx.alloc_array(2, 0);
    if arr_addr_0 != 0 {
        for elem_idx in 0..2u32 {
            let child_addr = pack_matrix(vm_context, val.projection[elem_idx as usize]);
            ctx.array_set_u64(arr_addr_0, elem_idx, child_addr as u64);
        }
        ctx.write_field_u64_by_index(addr, 0, arr_addr_0 as u64);
    }
    let arr_addr_1 = ctx.alloc_array(2, 0);
    if arr_addr_1 != 0 {
        for elem_idx in 0..2u32 {
            let child_addr = pack_matrix(vm_context, val.viewOffset[elem_idx as usize]);
            ctx.array_set_u64(arr_addr_1, elem_idx, child_addr as u64);
        }
        ctx.write_field_u64_by_index(addr, 1, arr_addr_1 as u64);
    }
    let arr_addr_2 = ctx.alloc_array(2, 0);
    if arr_addr_2 != 0 {
        for elem_idx in 0..2u32 {
            ctx.array_set_u64(arr_addr_2, elem_idx, (val.leftLensCenter[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 2, arr_addr_2 as u64);
    }
    let arr_addr_3 = ctx.alloc_array(2, 0);
    if arr_addr_3 != 0 {
        for elem_idx in 0..2u32 {
            ctx.array_set_u64(arr_addr_3, elem_idx, (val.rightLensCenter[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 3, arr_addr_3 as u64);
    }
    let arr_addr_4 = ctx.alloc_array(2, 0);
    if arr_addr_4 != 0 {
        for elem_idx in 0..2u32 {
            ctx.array_set_u64(arr_addr_4, elem_idx, (val.leftScreenCenter[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 4, arr_addr_4 as u64);
    }
    let arr_addr_5 = ctx.alloc_array(2, 0);
    if arr_addr_5 != 0 {
        for elem_idx in 0..2u32 {
            ctx.array_set_u64(arr_addr_5, elem_idx, (val.rightScreenCenter[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 5, arr_addr_5 as u64);
    }
    let arr_addr_6 = ctx.alloc_array(2, 0);
    if arr_addr_6 != 0 {
        for elem_idx in 0..2u32 {
            ctx.array_set_u64(arr_addr_6, elem_idx, (val.scale[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 6, arr_addr_6 as u64);
    }
    let arr_addr_7 = ctx.alloc_array(2, 0);
    if arr_addr_7 != 0 {
        for elem_idx in 0..2u32 {
            ctx.array_set_u64(arr_addr_7, elem_idx, (val.scaleIn[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 7, arr_addr_7 as u64);
    }
}

unsafe fn pack_vr_stereo_config(vm_context: *mut c_void, val: C_VrStereoConfig) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("VrStereoConfig");
    let addr = ctx.alloc_object(class_vtable_id, 8);
    if addr > 0 {
        writeback_vr_stereo_config(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_vr_stereo_config(vm_context: *mut c_void, addr: u32) -> C_VrStereoConfig {
    if addr == 0 { return C_VrStereoConfig::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_VrStereoConfig::default(); };
    let mut result = C_VrStereoConfig::default();
    let arr_addr_0 = ctx.read_field_u64_by_index(addr, 0) as u32;
    if arr_addr_0 > 0 {
        let arr_len = ctx.array_len(arr_addr_0).min(2);
        for elem_idx in 0..arr_len {
            let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
            result.projection[elem_idx as usize] = unpack_matrix(vm_context, child_addr);
        }
    }
    let arr_addr_1 = ctx.read_field_u64_by_index(addr, 1) as u32;
    if arr_addr_1 > 0 {
        let arr_len = ctx.array_len(arr_addr_1).min(2);
        for elem_idx in 0..arr_len {
            let child_addr = ctx.array_get_u64(arr_addr_1, elem_idx) as u32;
            result.viewOffset[elem_idx as usize] = unpack_matrix(vm_context, child_addr);
        }
    }
    let arr_addr_2 = ctx.read_field_u64_by_index(addr, 2) as u32;
    if arr_addr_2 > 0 {
        let arr_len = ctx.array_len(arr_addr_2).min(2);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
            result.leftLensCenter[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    let arr_addr_3 = ctx.read_field_u64_by_index(addr, 3) as u32;
    if arr_addr_3 > 0 {
        let arr_len = ctx.array_len(arr_addr_3).min(2);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_3, elem_idx);
            result.rightLensCenter[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    let arr_addr_4 = ctx.read_field_u64_by_index(addr, 4) as u32;
    if arr_addr_4 > 0 {
        let arr_len = ctx.array_len(arr_addr_4).min(2);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_4, elem_idx);
            result.leftScreenCenter[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    let arr_addr_5 = ctx.read_field_u64_by_index(addr, 5) as u32;
    if arr_addr_5 > 0 {
        let arr_len = ctx.array_len(arr_addr_5).min(2);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_5, elem_idx);
            result.rightScreenCenter[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    let arr_addr_6 = ctx.read_field_u64_by_index(addr, 6) as u32;
    if arr_addr_6 > 0 {
        let arr_len = ctx.array_len(arr_addr_6).min(2);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_6, elem_idx);
            result.scale[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    let arr_addr_7 = ctx.read_field_u64_by_index(addr, 7) as u32;
    if arr_addr_7 > 0 {
        let arr_len = ctx.array_len(arr_addr_7).min(2);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_7, elem_idx);
            result.scaleIn[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    result
}

unsafe fn writeback_file_path_list(vm_context: *mut c_void, addr: u32, val: &C_FilePathList) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.count as u64);
    ctx.write_field_u64_by_index(addr, 1, val.paths as usize as u64);
}

unsafe fn pack_file_path_list(vm_context: *mut c_void, val: C_FilePathList) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("FilePathList");
    let addr = ctx.alloc_object(class_vtable_id, 2);
    if addr > 0 {
        writeback_file_path_list(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_file_path_list(vm_context: *mut c_void, addr: u32) -> C_FilePathList {
    if addr == 0 { return C_FilePathList::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_FilePathList::default(); };
    let mut result = C_FilePathList::default();
    result.count = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.paths = ctx.read_field_u64_by_index(addr, 1) as usize as *mut *mut c_char;
    result
}

unsafe fn writeback_automation_event(vm_context: *mut c_void, addr: u32, val: &C_AutomationEvent) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.frame as u64);
    ctx.write_field_u64_by_index(addr, 1, val.r#type as u64);
    let arr_addr_2 = ctx.alloc_array(4, 0);
    if arr_addr_2 != 0 {
        for elem_idx in 0..4u32 {
            ctx.array_set_u64(arr_addr_2, elem_idx, val.params[elem_idx as usize] as u64);
        }
        ctx.write_field_u64_by_index(addr, 2, arr_addr_2 as u64);
    }
}

unsafe fn pack_automation_event(vm_context: *mut c_void, val: C_AutomationEvent) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("AutomationEvent");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_automation_event(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_automation_event(vm_context: *mut c_void, addr: u32) -> C_AutomationEvent {
    if addr == 0 { return C_AutomationEvent::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_AutomationEvent::default(); };
    let mut result = C_AutomationEvent::default();
    result.frame = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.r#type = ctx.read_field_u64_by_index(addr, 1) as c_uint;
    let arr_addr_2 = ctx.read_field_u64_by_index(addr, 2) as u32;
    if arr_addr_2 > 0 {
        let arr_len = ctx.array_len(arr_addr_2).min(4);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
            result.params[elem_idx as usize] = raw_val as c_int;
        }
    }
    result
}

unsafe fn writeback_automation_event_list(vm_context: *mut c_void, addr: u32, val: &C_AutomationEventList) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.write_field_u64_by_index(addr, 0, val.capacity as u64);
    ctx.write_field_u64_by_index(addr, 1, val.count as u64);
    ctx.write_field_u64_by_index(addr, 2, val.events as usize as u64);
}

unsafe fn pack_automation_event_list(vm_context: *mut c_void, val: C_AutomationEventList) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("AutomationEventList");
    let addr = ctx.alloc_object(class_vtable_id, 3);
    if addr > 0 {
        writeback_automation_event_list(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_automation_event_list(vm_context: *mut c_void, addr: u32) -> C_AutomationEventList {
    if addr == 0 { return C_AutomationEventList::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_AutomationEventList::default(); };
    let mut result = C_AutomationEventList::default();
    result.capacity = ctx.read_field_u64_by_index(addr, 0) as c_uint;
    result.count = ctx.read_field_u64_by_index(addr, 1) as c_uint;
    result.events = ctx.read_field_u64_by_index(addr, 2) as usize as *mut C_AutomationEvent;
    result
}

unsafe fn writeback_rl_vertex_buffer(vm_context: *mut c_void, addr: u32, val: &C_rlVertexBuffer) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.elementCount as i32);
    ctx.write_field_u64_by_index(addr, 1, val.vertices as usize as u64);
    ctx.write_field_u64_by_index(addr, 2, val.texcoords as usize as u64);
    ctx.write_field_u64_by_index(addr, 3, val.normals as usize as u64);
    ctx.write_field_u64_by_index(addr, 4, val.colors as usize as u64);
    ctx.write_field_u64_by_index(addr, 5, val.indices as usize as u64);
    ctx.write_field_u64_by_index(addr, 6, val.vaoId as u64);
    let arr_addr_7 = ctx.alloc_array(5, 0);
    if arr_addr_7 != 0 {
        for elem_idx in 0..5u32 {
            ctx.array_set_u64(arr_addr_7, elem_idx, val.vboId[elem_idx as usize] as u64);
        }
        ctx.write_field_u64_by_index(addr, 7, arr_addr_7 as u64);
    }
}

unsafe fn pack_rl_vertex_buffer(vm_context: *mut c_void, val: C_rlVertexBuffer) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("rlVertexBuffer");
    let addr = ctx.alloc_object(class_vtable_id, 8);
    if addr > 0 {
        writeback_rl_vertex_buffer(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_rl_vertex_buffer(vm_context: *mut c_void, addr: u32) -> C_rlVertexBuffer {
    if addr == 0 { return C_rlVertexBuffer::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_rlVertexBuffer::default(); };
    let mut result = C_rlVertexBuffer::default();
    result.elementCount = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.vertices = ctx.read_field_u64_by_index(addr, 1) as usize as *mut f32;
    result.texcoords = ctx.read_field_u64_by_index(addr, 2) as usize as *mut f32;
    result.normals = ctx.read_field_u64_by_index(addr, 3) as usize as *mut f32;
    result.colors = ctx.read_field_u64_by_index(addr, 4) as usize as *mut u8;
    result.indices = ctx.read_field_u64_by_index(addr, 5) as usize as *mut c_uint;
    result.vaoId = ctx.read_field_u64_by_index(addr, 6) as c_uint;
    let arr_addr_7 = ctx.read_field_u64_by_index(addr, 7) as u32;
    if arr_addr_7 > 0 {
        let arr_len = ctx.array_len(arr_addr_7).min(5);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_7, elem_idx);
            result.vboId[elem_idx as usize] = raw_val as c_uint;
        }
    }
    result
}

unsafe fn writeback_rl_draw_call(vm_context: *mut c_void, addr: u32, val: &C_rlDrawCall) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.mode as i32);
    ctx.set_field_i32_by_index(addr, 1, val.vertexCount as i32);
    ctx.set_field_i32_by_index(addr, 2, val.vertexAlignment as i32);
    ctx.write_field_u64_by_index(addr, 3, val.textureId as u64);
}

unsafe fn pack_rl_draw_call(vm_context: *mut c_void, val: C_rlDrawCall) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("rlDrawCall");
    let addr = ctx.alloc_object(class_vtable_id, 4);
    if addr > 0 {
        writeback_rl_draw_call(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_rl_draw_call(vm_context: *mut c_void, addr: u32) -> C_rlDrawCall {
    if addr == 0 { return C_rlDrawCall::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_rlDrawCall::default(); };
    let mut result = C_rlDrawCall::default();
    result.mode = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.vertexCount = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.vertexAlignment = ctx.get_field_i32_by_index(addr, 2) as c_int;
    result.textureId = ctx.read_field_u64_by_index(addr, 3) as c_uint;
    result
}

unsafe fn writeback_rl_render_batch(vm_context: *mut c_void, addr: u32, val: &C_rlRenderBatch) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    ctx.set_field_i32_by_index(addr, 0, val.bufferCount as i32);
    ctx.set_field_i32_by_index(addr, 1, val.currentBuffer as i32);
    ctx.write_field_u64_by_index(addr, 2, val.vertexBuffer as usize as u64);
    ctx.write_field_u64_by_index(addr, 3, val.draws as usize as u64);
    ctx.set_field_i32_by_index(addr, 4, val.drawCounter as i32);
    ctx.write_field_u64_by_index(addr, 5, val.currentDepth.to_bits() as u64);
}

unsafe fn pack_rl_render_batch(vm_context: *mut c_void, val: C_rlRenderBatch) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("rlRenderBatch");
    let addr = ctx.alloc_object(class_vtable_id, 6);
    if addr > 0 {
        writeback_rl_render_batch(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_rl_render_batch(vm_context: *mut c_void, addr: u32) -> C_rlRenderBatch {
    if addr == 0 { return C_rlRenderBatch::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_rlRenderBatch::default(); };
    let mut result = C_rlRenderBatch::default();
    result.bufferCount = ctx.get_field_i32_by_index(addr, 0) as c_int;
    result.currentBuffer = ctx.get_field_i32_by_index(addr, 1) as c_int;
    result.vertexBuffer = ctx.read_field_u64_by_index(addr, 2) as usize as *mut C_rlVertexBuffer;
    result.draws = ctx.read_field_u64_by_index(addr, 3) as usize as *mut C_rlDrawCall;
    result.drawCounter = ctx.get_field_i32_by_index(addr, 4) as c_int;
    result.currentDepth = f32::from_bits(ctx.read_field_u64_by_index(addr, 5) as u32);
    result
}

unsafe fn writeback_float3(vm_context: *mut c_void, addr: u32, val: &C_float3) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let arr_addr_0 = ctx.alloc_array(3, 0);
    if arr_addr_0 != 0 {
        for elem_idx in 0..3u32 {
            ctx.array_set_u64(arr_addr_0, elem_idx, (val.v[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 0, arr_addr_0 as u64);
    }
}

unsafe fn pack_float3(vm_context: *mut c_void, val: C_float3) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("float3");
    let addr = ctx.alloc_object(class_vtable_id, 1);
    if addr > 0 {
        writeback_float3(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_float3(vm_context: *mut c_void, addr: u32) -> C_float3 {
    if addr == 0 { return C_float3::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_float3::default(); };
    let mut result = C_float3::default();
    let arr_addr_0 = ctx.read_field_u64_by_index(addr, 0) as u32;
    if arr_addr_0 > 0 {
        let arr_len = ctx.array_len(arr_addr_0).min(3);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
            result.v[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
    result
}

unsafe fn writeback_float16(vm_context: *mut c_void, addr: u32, val: &C_float16) {
    if addr == 0 { return; }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return; };
    let arr_addr_0 = ctx.alloc_array(16, 0);
    if arr_addr_0 != 0 {
        for elem_idx in 0..16u32 {
            ctx.array_set_u64(arr_addr_0, elem_idx, (val.v[elem_idx as usize] as f64).to_bits());
        }
        ctx.write_field_u64_by_index(addr, 0, arr_addr_0 as u64);
    }
}

unsafe fn pack_float16(vm_context: *mut c_void, val: C_float16) -> u32 {
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return 0; };
    let class_vtable_id = ctx.get_class_vtable_id("float16");
    let addr = ctx.alloc_object(class_vtable_id, 1);
    if addr > 0 {
        writeback_float16(vm_context, addr, &val);
    }
    addr
}

unsafe fn unpack_float16(vm_context: *mut c_void, addr: u32) -> C_float16 {
    if addr == 0 { return C_float16::default(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return C_float16::default(); };
    let mut result = C_float16::default();
    let arr_addr_0 = ctx.read_field_u64_by_index(addr, 0) as u32;
    if arr_addr_0 > 0 {
        let arr_len = ctx.array_len(arr_addr_0).min(16);
        for elem_idx in 0..arr_len {
            let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
            result.v[elem_idx as usize] = if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 };
        }
    }
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
// Upstream Extern "C" Declarations
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
    pub fn BeginTextureMode(arg_0: C_RenderTexture);
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
    pub fn SetShaderValueTexture(arg_0: C_Shader, arg_1: c_int, arg_2: C_Texture);
    pub fn UnloadShader(arg_0: C_Shader);
    pub fn GetScreenToWorldRay(arg_0: C_Vector2, arg_1: C_Camera3D) -> C_Ray;
    pub fn GetScreenToWorldRayEx(arg_0: C_Vector2, arg_1: C_Camera3D, arg_2: c_int, arg_3: c_int) -> C_Ray;
    pub fn GetWorldToScreen(arg_0: C_Vector3, arg_1: C_Camera3D) -> C_Vector2;
    pub fn GetWorldToScreenEx(arg_0: C_Vector3, arg_1: C_Camera3D, arg_2: c_int, arg_3: c_int) -> C_Vector2;
    pub fn GetWorldToScreen2D(arg_0: C_Vector2, arg_1: C_Camera2D) -> C_Vector2;
    pub fn GetScreenToWorld2D(arg_0: C_Vector2, arg_1: C_Camera2D) -> C_Vector2;
    pub fn GetCameraMatrix(arg_0: C_Camera3D) -> C_Matrix;
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
    pub fn ExportDataAsCode(arg_0: *const u8, arg_1: c_int, arg_2: *const c_char) -> bool;
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
    pub fn CompressData(arg_0: *const u8, arg_1: c_int, arg_2: *mut c_int) -> *mut u8;
    pub fn DecompressData(arg_0: *const u8, arg_1: c_int, arg_2: *mut c_int) -> *mut u8;
    pub fn EncodeDataBase64(arg_0: *const u8, arg_1: c_int, arg_2: *mut c_int) -> *const c_char;
    pub fn DecodeDataBase64(arg_0: *const c_char, arg_1: *mut c_int) -> *mut u8;
    pub fn ComputeCRC32(arg_0: *const u8, arg_1: c_int) -> c_uint;
    pub fn ComputeMD5(arg_0: *const u8, arg_1: c_int) -> *mut c_uint;
    pub fn ComputeSHA1(arg_0: *const u8, arg_1: c_int) -> *mut c_uint;
    pub fn ComputeSHA256(arg_0: *const u8, arg_1: c_int) -> *mut c_uint;
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
    pub fn UpdateCamera(arg_0: *mut C_Camera3D, arg_1: c_int);
    pub fn UpdateCameraPro(arg_0: *mut C_Camera3D, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: f32);
    pub fn SetShapesTexture(arg_0: C_Texture, arg_1: C_Rectangle);
    pub fn GetShapesTexture() -> C_Texture;
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
    pub fn LoadImageAnimFromMemory(arg_0: *const c_char, arg_1: *const u8, arg_2: c_int, arg_3: *mut c_int) -> C_Image;
    pub fn LoadImageFromMemory(arg_0: *const c_char, arg_1: *const u8, arg_2: c_int) -> C_Image;
    pub fn LoadImageFromTexture(arg_0: C_Texture) -> C_Image;
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
    pub fn ImageKernelConvolution(arg_0: *mut C_Image, arg_1: *const f32, arg_2: c_int);
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
    pub fn LoadTexture(arg_0: *const c_char) -> C_Texture;
    pub fn LoadTextureFromImage(arg_0: C_Image) -> C_Texture;
    pub fn LoadTextureCubemap(arg_0: C_Image, arg_1: c_int) -> C_Texture;
    pub fn LoadRenderTexture(arg_0: c_int, arg_1: c_int) -> C_RenderTexture;
    pub fn LoadRenderTextureEx(arg_0: c_int, arg_1: c_int, arg_2: c_int) -> C_RenderTexture;
    pub fn IsTextureValid(arg_0: C_Texture) -> bool;
    pub fn UnloadTexture(arg_0: C_Texture);
    pub fn IsRenderTextureValid(arg_0: C_RenderTexture) -> bool;
    pub fn UnloadRenderTexture(arg_0: C_RenderTexture);
    pub fn UpdateTexture(arg_0: C_Texture, arg_1: *mut c_void);
    pub fn UpdateTextureRec(arg_0: C_Texture, arg_1: C_Rectangle, arg_2: *mut c_void);
    pub fn GenTextureMipmaps(arg_0: *mut C_Texture);
    pub fn SetTextureFilter(arg_0: C_Texture, arg_1: c_int);
    pub fn SetTextureWrap(arg_0: C_Texture, arg_1: c_int);
    pub fn DrawTexture(arg_0: C_Texture, arg_1: c_int, arg_2: c_int, arg_3: C_Color);
    pub fn DrawTextureV(arg_0: C_Texture, arg_1: C_Vector2, arg_2: C_Color);
    pub fn DrawTextureEx(arg_0: C_Texture, arg_1: C_Vector2, arg_2: f32, arg_3: f32, arg_4: C_Color);
    pub fn DrawTextureRec(arg_0: C_Texture, arg_1: C_Rectangle, arg_2: C_Vector2, arg_3: C_Color);
    pub fn DrawTexturePro(arg_0: C_Texture, arg_1: C_Rectangle, arg_2: C_Rectangle, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
    pub fn DrawTextureNPatch(arg_0: C_Texture, arg_1: C_NPatchInfo, arg_2: C_Rectangle, arg_3: C_Vector2, arg_4: f32, arg_5: C_Color);
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
    pub fn LoadFontEx(arg_0: *const c_char, arg_1: c_int, arg_2: *const c_int, arg_3: c_int) -> C_Font;
    pub fn LoadFontFromImage(arg_0: C_Image, arg_1: C_Color, arg_2: c_int) -> C_Font;
    pub fn LoadFontFromMemory(arg_0: *const c_char, arg_1: *const u8, arg_2: c_int, arg_3: c_int, arg_4: *const c_int, arg_5: c_int) -> C_Font;
    pub fn IsFontValid(arg_0: C_Font) -> bool;
    pub fn LoadFontData(arg_0: *const u8, arg_1: c_int, arg_2: c_int, arg_3: *const c_int, arg_4: c_int, arg_5: c_int, arg_6: *mut c_int) -> *mut C_GlyphInfo;
    pub fn GenImageFontAtlas(arg_0: *const C_GlyphInfo, arg_1: *mut c_void, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: c_int) -> C_Image;
    pub fn UnloadFontData(arg_0: *mut C_GlyphInfo, arg_1: c_int);
    pub fn UnloadFont(arg_0: C_Font);
    pub fn ExportFontAsCode(arg_0: C_Font, arg_1: *const c_char) -> bool;
    pub fn DrawFPS(arg_0: c_int, arg_1: c_int);
    pub fn DrawText(arg_0: *const c_char, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: C_Color);
    pub fn DrawTextEx(arg_0: C_Font, arg_1: *const c_char, arg_2: C_Vector2, arg_3: f32, arg_4: f32, arg_5: C_Color);
    pub fn DrawTextPro(arg_0: C_Font, arg_1: *const c_char, arg_2: C_Vector2, arg_3: C_Vector2, arg_4: f32, arg_5: f32, arg_6: f32, arg_7: C_Color);
    pub fn DrawTextCodepoint(arg_0: C_Font, arg_1: c_int, arg_2: C_Vector2, arg_3: f32, arg_4: C_Color);
    pub fn DrawTextCodepoints(arg_0: C_Font, arg_1: *const c_int, arg_2: c_int, arg_3: C_Vector2, arg_4: f32, arg_5: f32, arg_6: C_Color);
    pub fn SetTextLineSpacing(arg_0: c_int);
    pub fn MeasureText(arg_0: *const c_char, arg_1: c_int) -> c_int;
    pub fn MeasureTextEx(arg_0: C_Font, arg_1: *const c_char, arg_2: f32, arg_3: f32) -> C_Vector2;
    pub fn MeasureTextCodepoints(arg_0: C_Font, arg_1: *const c_int, arg_2: c_int, arg_3: f32, arg_4: f32) -> C_Vector2;
    pub fn GetGlyphIndex(arg_0: C_Font, arg_1: c_int) -> c_int;
    pub fn GetGlyphInfo(arg_0: C_Font, arg_1: c_int) -> C_GlyphInfo;
    pub fn GetGlyphAtlasRec(arg_0: C_Font, arg_1: c_int) -> C_Rectangle;
    pub fn LoadUTF8(arg_0: *const c_int, arg_1: c_int) -> *const c_char;
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
    pub fn DrawBillboard(arg_0: C_Camera3D, arg_1: C_Texture, arg_2: C_Vector3, arg_3: f32, arg_4: C_Color);
    pub fn DrawBillboardRec(arg_0: C_Camera3D, arg_1: C_Texture, arg_2: C_Rectangle, arg_3: C_Vector3, arg_4: C_Vector2, arg_5: C_Color);
    pub fn DrawBillboardPro(arg_0: C_Camera3D, arg_1: C_Texture, arg_2: C_Rectangle, arg_3: C_Vector3, arg_4: C_Vector3, arg_5: C_Vector2, arg_6: C_Vector2, arg_7: f32, arg_8: C_Color);
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
    pub fn SetMaterialTexture(arg_0: *mut C_Material, arg_1: c_int, arg_2: C_Texture);
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
    pub fn LoadWaveFromMemory(arg_0: *const c_char, arg_1: *const u8, arg_2: c_int) -> C_Wave;
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
    pub fn LoadMusicStreamFromMemory(arg_0: *const c_char, arg_1: *const u8, arg_2: c_int) -> C_Music;
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
    pub fn rlMatrixMode(arg_0: c_int);
    pub fn rlPushMatrix();
    pub fn rlPopMatrix();
    pub fn rlLoadIdentity();
    pub fn rlTranslatef(arg_0: f32, arg_1: f32, arg_2: f32);
    pub fn rlRotatef(arg_0: f32, arg_1: f32, arg_2: f32, arg_3: f32);
    pub fn rlScalef(arg_0: f32, arg_1: f32, arg_2: f32);
    pub fn rlMultMatrixf(arg_0: *const f32);
    pub fn rlFrustum(arg_0: f64, arg_1: f64, arg_2: f64, arg_3: f64, arg_4: f64, arg_5: f64);
    pub fn rlOrtho(arg_0: f64, arg_1: f64, arg_2: f64, arg_3: f64, arg_4: f64, arg_5: f64);
    pub fn rlViewport(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int);
    pub fn rlSetClipPlanes(arg_0: f64, arg_1: f64);
    pub fn rlGetCullDistanceNear() -> f64;
    pub fn rlGetCullDistanceFar() -> f64;
    pub fn rlBegin(arg_0: c_int);
    pub fn rlEnd();
    pub fn rlVertex2i(arg_0: c_int, arg_1: c_int);
    pub fn rlVertex2f(arg_0: f32, arg_1: f32);
    pub fn rlVertex3f(arg_0: f32, arg_1: f32, arg_2: f32);
    pub fn rlTexCoord2f(arg_0: f32, arg_1: f32);
    pub fn rlNormal3f(arg_0: f32, arg_1: f32, arg_2: f32);
    pub fn rlColor4ub(arg_0: u8, arg_1: u8, arg_2: u8, arg_3: u8);
    pub fn rlColor3f(arg_0: f32, arg_1: f32, arg_2: f32);
    pub fn rlColor4f(arg_0: f32, arg_1: f32, arg_2: f32, arg_3: f32);
    pub fn rlEnableVertexArray(arg_0: c_uint) -> bool;
    pub fn rlDisableVertexArray();
    pub fn rlEnableVertexBuffer(arg_0: c_uint);
    pub fn rlDisableVertexBuffer();
    pub fn rlEnableVertexBufferElement(arg_0: c_uint);
    pub fn rlDisableVertexBufferElement();
    pub fn rlEnableVertexAttribute(arg_0: c_uint);
    pub fn rlDisableVertexAttribute(arg_0: c_uint);
    pub fn rlEnableStatePointer(arg_0: c_int, arg_1: *mut c_void);
    pub fn rlDisableStatePointer(arg_0: c_int);
    pub fn rlActiveTextureSlot(arg_0: c_int);
    pub fn rlEnableTexture(arg_0: c_uint);
    pub fn rlDisableTexture();
    pub fn rlEnableTextureCubemap(arg_0: c_uint);
    pub fn rlDisableTextureCubemap();
    pub fn rlTextureParameters(arg_0: c_uint, arg_1: c_int, arg_2: c_int);
    pub fn rlCubemapParameters(arg_0: c_uint, arg_1: c_int, arg_2: c_int);
    pub fn rlEnableShader(arg_0: c_uint);
    pub fn rlDisableShader();
    pub fn rlEnableFramebuffer(arg_0: c_uint);
    pub fn rlDisableFramebuffer();
    pub fn rlGetActiveFramebuffer() -> c_uint;
    pub fn rlActiveDrawBuffers(arg_0: c_int);
    pub fn rlBlitFramebuffer(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: c_int, arg_6: c_int, arg_7: c_int, arg_8: c_int);
    pub fn rlBindFramebuffer(arg_0: c_uint, arg_1: c_uint);
    pub fn rlEnableColorBlend();
    pub fn rlDisableColorBlend();
    pub fn rlEnableDepthTest();
    pub fn rlDisableDepthTest();
    pub fn rlEnableDepthMask();
    pub fn rlDisableDepthMask();
    pub fn rlEnableBackfaceCulling();
    pub fn rlDisableBackfaceCulling();
    pub fn rlColorMask(arg_0: bool, arg_1: bool, arg_2: bool, arg_3: bool);
    pub fn rlSetCullFace(arg_0: c_int);
    pub fn rlEnableScissorTest();
    pub fn rlDisableScissorTest();
    pub fn rlScissor(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int);
    pub fn rlEnablePointMode();
    pub fn rlDisablePointMode();
    pub fn rlSetPointSize(arg_0: f32);
    pub fn rlGetPointSize() -> f32;
    pub fn rlEnableWireMode();
    pub fn rlDisableWireMode();
    pub fn rlSetLineWidth(arg_0: f32);
    pub fn rlGetLineWidth() -> f32;
    pub fn rlEnableSmoothLines();
    pub fn rlDisableSmoothLines();
    pub fn rlEnableStereoRender();
    pub fn rlDisableStereoRender();
    pub fn rlIsStereoRenderEnabled() -> bool;
    pub fn rlClearColor(arg_0: u8, arg_1: u8, arg_2: u8, arg_3: u8);
    pub fn rlClearScreenBuffers();
    pub fn rlCheckErrors();
    pub fn rlSetBlendMode(arg_0: c_int);
    pub fn rlSetBlendFactors(arg_0: c_int, arg_1: c_int, arg_2: c_int);
    pub fn rlSetBlendFactorsSeparate(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: c_int);
    pub fn rlglInit(arg_0: c_int, arg_1: c_int);
    pub fn rlglClose();
    pub fn rlLoadExtensions(arg_0: *mut c_void);
    pub fn rlGetProcAddress(arg_0: *const c_char) -> *mut c_void;
    pub fn rlGetVersion() -> c_int;
    pub fn rlSetFramebufferWidth(arg_0: c_int);
    pub fn rlGetFramebufferWidth() -> c_int;
    pub fn rlSetFramebufferHeight(arg_0: c_int);
    pub fn rlGetFramebufferHeight() -> c_int;
    pub fn rlGetTextureIdDefault() -> c_uint;
    pub fn rlGetShaderIdDefault() -> c_uint;
    pub fn rlGetShaderLocsDefault() -> *mut c_int;
    pub fn rlLoadRenderBatch(arg_0: c_int, arg_1: c_int) -> C_rlRenderBatch;
    pub fn rlUnloadRenderBatch(arg_0: C_rlRenderBatch);
    pub fn rlDrawRenderBatch(arg_0: *mut C_rlRenderBatch);
    pub fn rlSetRenderBatchActive(arg_0: *mut C_rlRenderBatch);
    pub fn rlDrawRenderBatchActive();
    pub fn rlCheckRenderBatchLimit(arg_0: c_int) -> bool;
    pub fn rlSetTexture(arg_0: c_uint);
    pub fn rlLoadVertexArray() -> c_uint;
    pub fn rlLoadVertexBuffer(arg_0: *mut c_void, arg_1: c_int, arg_2: bool) -> c_uint;
    pub fn rlLoadVertexBufferElement(arg_0: *mut c_void, arg_1: c_int, arg_2: bool) -> c_uint;
    pub fn rlUpdateVertexBuffer(arg_0: c_uint, arg_1: *mut c_void, arg_2: c_int, arg_3: c_int);
    pub fn rlUpdateVertexBufferElements(arg_0: c_uint, arg_1: *mut c_void, arg_2: c_int, arg_3: c_int);
    pub fn rlUnloadVertexArray(arg_0: c_uint);
    pub fn rlUnloadVertexBuffer(arg_0: c_uint);
    pub fn rlSetVertexAttribute(arg_0: c_uint, arg_1: c_int, arg_2: c_int, arg_3: bool, arg_4: c_int, arg_5: c_int);
    pub fn rlSetVertexAttributeDivisor(arg_0: c_uint, arg_1: c_int);
    pub fn rlSetVertexAttributeDefault(arg_0: c_int, arg_1: *mut c_void, arg_2: c_int, arg_3: c_int);
    pub fn rlDrawVertexArray(arg_0: c_int, arg_1: c_int);
    pub fn rlDrawVertexArrayElements(arg_0: c_int, arg_1: c_int, arg_2: *mut c_void);
    pub fn rlDrawVertexArrayInstanced(arg_0: c_int, arg_1: c_int, arg_2: c_int);
    pub fn rlDrawVertexArrayElementsInstanced(arg_0: c_int, arg_1: c_int, arg_2: *mut c_void, arg_3: c_int);
    pub fn rlLoadTexture(arg_0: *mut c_void, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int) -> c_uint;
    pub fn rlLoadTextureDepth(arg_0: c_int, arg_1: c_int, arg_2: bool) -> c_uint;
    pub fn rlLoadTextureCubemap(arg_0: *mut c_void, arg_1: c_int, arg_2: c_int, arg_3: c_int) -> c_uint;
    pub fn rlUpdateTexture(arg_0: c_uint, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: c_int, arg_6: *mut c_void);
    pub fn rlGetGlTextureFormats(arg_0: c_int, arg_1: *mut c_uint, arg_2: *mut c_uint, arg_3: *mut c_uint);
    pub fn rlGetPixelFormatName(arg_0: c_uint) -> *const c_char;
    pub fn rlUnloadTexture(arg_0: c_uint);
    pub fn rlGenTextureMipmaps(arg_0: c_uint, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: *mut c_int);
    pub fn rlReadTexturePixels(arg_0: c_uint, arg_1: c_int, arg_2: c_int, arg_3: c_int) -> *mut c_void;
    pub fn rlReadScreenPixels(arg_0: c_int, arg_1: c_int) -> *mut u8;
    pub fn rlLoadFramebuffer() -> c_uint;
    pub fn rlFramebufferAttach(arg_0: c_uint, arg_1: c_uint, arg_2: c_int, arg_3: c_int, arg_4: c_int);
    pub fn rlFramebufferComplete(arg_0: c_uint) -> bool;
    pub fn rlUnloadFramebuffer(arg_0: c_uint);
    pub fn rlCopyFramebuffer(arg_0: c_int, arg_1: c_int, arg_2: c_int, arg_3: c_int, arg_4: c_int, arg_5: *mut c_void);
    pub fn rlResizeFramebuffer(arg_0: c_int, arg_1: c_int);
    pub fn rlLoadShader(arg_0: *const c_char, arg_1: c_int) -> c_uint;
    pub fn rlLoadShaderProgram(arg_0: *const c_char, arg_1: *const c_char) -> c_uint;
    pub fn rlLoadShaderProgramEx(arg_0: c_uint, arg_1: c_uint) -> c_uint;
    pub fn rlLoadShaderProgramCompute(arg_0: c_uint) -> c_uint;
    pub fn rlUnloadShader(arg_0: c_uint);
    pub fn rlUnloadShaderProgram(arg_0: c_uint);
    pub fn rlGetLocationUniform(arg_0: c_uint, arg_1: *const c_char) -> c_int;
    pub fn rlGetLocationAttrib(arg_0: c_uint, arg_1: *const c_char) -> c_int;
    pub fn rlSetUniform(arg_0: c_int, arg_1: *mut c_void, arg_2: c_int, arg_3: c_int);
    pub fn rlSetUniformMatrix(arg_0: c_int, arg_1: C_Matrix);
    pub fn rlSetUniformMatrices(arg_0: c_int, arg_1: *const C_Matrix, arg_2: c_int);
    pub fn rlSetUniformSampler(arg_0: c_int, arg_1: c_uint);
    pub fn rlSetShader(arg_0: c_uint, arg_1: *mut c_int);
    pub fn rlComputeShaderDispatch(arg_0: c_uint, arg_1: c_uint, arg_2: c_uint);
    pub fn rlLoadShaderBuffer(arg_0: c_uint, arg_1: *mut c_void, arg_2: c_int) -> c_uint;
    pub fn rlUnloadShaderBuffer(arg_0: c_uint);
    pub fn rlUpdateShaderBuffer(arg_0: c_uint, arg_1: *mut c_void, arg_2: c_uint, arg_3: c_uint);
    pub fn rlBindShaderBuffer(arg_0: c_uint, arg_1: c_uint);
    pub fn rlReadShaderBuffer(arg_0: c_uint, arg_1: *mut c_void, arg_2: c_uint, arg_3: c_uint);
    pub fn rlCopyShaderBuffer(arg_0: c_uint, arg_1: c_uint, arg_2: c_uint, arg_3: c_uint, arg_4: c_uint);
    pub fn rlGetShaderBufferSize(arg_0: c_uint) -> c_uint;
    pub fn rlBindImageTexture(arg_0: c_uint, arg_1: c_uint, arg_2: c_int, arg_3: bool);
    pub fn rlGetMatrixModelview() -> C_Matrix;
    pub fn rlGetMatrixProjection() -> C_Matrix;
    pub fn rlGetMatrixTransform() -> C_Matrix;
    pub fn rlGetMatrixProjectionStereo(arg_0: c_int) -> C_Matrix;
    pub fn rlGetMatrixViewOffsetStereo(arg_0: c_int) -> C_Matrix;
    pub fn rlSetMatrixProjection(arg_0: C_Matrix);
    pub fn rlSetMatrixModelview(arg_0: C_Matrix);
    pub fn rlSetMatrixProjectionStereo(arg_0: C_Matrix, arg_1: C_Matrix);
    pub fn rlSetMatrixViewOffsetStereo(arg_0: C_Matrix, arg_1: C_Matrix);
    pub fn rlLoadDrawCube();
    pub fn rlLoadDrawQuad();
    pub fn Clamp(arg_0: f32, arg_1: f32, arg_2: f32) -> f32;
    pub fn Lerp(arg_0: f32, arg_1: f32, arg_2: f32) -> f32;
    pub fn Normalize(arg_0: f32, arg_1: f32, arg_2: f32) -> f32;
    pub fn Remap(arg_0: f32, arg_1: f32, arg_2: f32, arg_3: f32, arg_4: f32) -> f32;
    pub fn Wrap(arg_0: f32, arg_1: f32, arg_2: f32) -> f32;
    pub fn FloatEquals(arg_0: f32, arg_1: f32) -> c_int;
    pub fn Vector2Zero() -> C_Vector2;
    pub fn Vector2One() -> C_Vector2;
    pub fn Vector2Add(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2AddValue(arg_0: C_Vector2, arg_1: f32) -> C_Vector2;
    pub fn Vector2Subtract(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2SubtractValue(arg_0: C_Vector2, arg_1: f32) -> C_Vector2;
    pub fn Vector2Length(arg_0: C_Vector2) -> f32;
    pub fn Vector2LengthSqr(arg_0: C_Vector2) -> f32;
    pub fn Vector2DotProduct(arg_0: C_Vector2, arg_1: C_Vector2) -> f32;
    pub fn Vector2CrossProduct(arg_0: C_Vector2, arg_1: C_Vector2) -> f32;
    pub fn Vector2Distance(arg_0: C_Vector2, arg_1: C_Vector2) -> f32;
    pub fn Vector2DistanceSqr(arg_0: C_Vector2, arg_1: C_Vector2) -> f32;
    pub fn Vector2Angle(arg_0: C_Vector2, arg_1: C_Vector2) -> f32;
    pub fn Vector2LineAngle(arg_0: C_Vector2, arg_1: C_Vector2) -> f32;
    pub fn Vector2Scale(arg_0: C_Vector2, arg_1: f32) -> C_Vector2;
    pub fn Vector2Multiply(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2Negate(arg_0: C_Vector2) -> C_Vector2;
    pub fn Vector2Divide(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2Normalize(arg_0: C_Vector2) -> C_Vector2;
    pub fn Vector2Transform(arg_0: C_Vector2, arg_1: C_Matrix) -> C_Vector2;
    pub fn Vector2Lerp(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32) -> C_Vector2;
    pub fn Vector2Reflect(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2Min(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2Max(arg_0: C_Vector2, arg_1: C_Vector2) -> C_Vector2;
    pub fn Vector2Rotate(arg_0: C_Vector2, arg_1: f32) -> C_Vector2;
    pub fn Vector2MoveTowards(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32) -> C_Vector2;
    pub fn Vector2Invert(arg_0: C_Vector2) -> C_Vector2;
    pub fn Vector2Clamp(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: C_Vector2) -> C_Vector2;
    pub fn Vector2ClampValue(arg_0: C_Vector2, arg_1: f32, arg_2: f32) -> C_Vector2;
    pub fn Vector2Equals(arg_0: C_Vector2, arg_1: C_Vector2) -> c_int;
    pub fn Vector2Refract(arg_0: C_Vector2, arg_1: C_Vector2, arg_2: f32) -> C_Vector2;
    pub fn Vector3Zero() -> C_Vector3;
    pub fn Vector3One() -> C_Vector3;
    pub fn Vector3Add(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3AddValue(arg_0: C_Vector3, arg_1: f32) -> C_Vector3;
    pub fn Vector3Subtract(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3SubtractValue(arg_0: C_Vector3, arg_1: f32) -> C_Vector3;
    pub fn Vector3Scale(arg_0: C_Vector3, arg_1: f32) -> C_Vector3;
    pub fn Vector3Multiply(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3CrossProduct(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3Perpendicular(arg_0: C_Vector3) -> C_Vector3;
    pub fn Vector3Length(arg_0: C_Vector3) -> f32;
    pub fn Vector3LengthSqr(arg_0: C_Vector3) -> f32;
    pub fn Vector3DotProduct(arg_0: C_Vector3, arg_1: C_Vector3) -> f32;
    pub fn Vector3Distance(arg_0: C_Vector3, arg_1: C_Vector3) -> f32;
    pub fn Vector3DistanceSqr(arg_0: C_Vector3, arg_1: C_Vector3) -> f32;
    pub fn Vector3Angle(arg_0: C_Vector3, arg_1: C_Vector3) -> f32;
    pub fn Vector3Negate(arg_0: C_Vector3) -> C_Vector3;
    pub fn Vector3Divide(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3Normalize(arg_0: C_Vector3) -> C_Vector3;
    pub fn Vector3Project(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3Reject(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3OrthoNormalize(arg_0: *mut C_Vector3, arg_1: *mut C_Vector3);
    pub fn Vector3Transform(arg_0: C_Vector3, arg_1: C_Matrix) -> C_Vector3;
    pub fn Vector3RotateByQuaternion(arg_0: C_Vector3, arg_1: C_Vector4) -> C_Vector3;
    pub fn Vector3RotateByAxisAngle(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32) -> C_Vector3;
    pub fn Vector3MoveTowards(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32) -> C_Vector3;
    pub fn Vector3Lerp(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32) -> C_Vector3;
    pub fn Vector3CubicHermite(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: C_Vector3, arg_4: f32) -> C_Vector3;
    pub fn Vector3Reflect(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3Min(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3Max(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector3;
    pub fn Vector3Barycenter(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Vector3, arg_3: C_Vector3) -> C_Vector3;
    pub fn Vector3Unproject(arg_0: C_Vector3, arg_1: C_Matrix, arg_2: C_Matrix) -> C_Vector3;
    pub fn Vector3ToFloatV(arg_0: C_Vector3) -> C_float3;
    pub fn Vector3Invert(arg_0: C_Vector3) -> C_Vector3;
    pub fn Vector3Clamp(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Vector3) -> C_Vector3;
    pub fn Vector3ClampValue(arg_0: C_Vector3, arg_1: f32, arg_2: f32) -> C_Vector3;
    pub fn Vector3Equals(arg_0: C_Vector3, arg_1: C_Vector3) -> c_int;
    pub fn Vector3Refract(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: f32) -> C_Vector3;
    pub fn Vector4Zero() -> C_Vector4;
    pub fn Vector4One() -> C_Vector4;
    pub fn Vector4Add(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn Vector4AddValue(arg_0: C_Vector4, arg_1: f32) -> C_Vector4;
    pub fn Vector4Subtract(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn Vector4SubtractValue(arg_0: C_Vector4, arg_1: f32) -> C_Vector4;
    pub fn Vector4Length(arg_0: C_Vector4) -> f32;
    pub fn Vector4LengthSqr(arg_0: C_Vector4) -> f32;
    pub fn Vector4DotProduct(arg_0: C_Vector4, arg_1: C_Vector4) -> f32;
    pub fn Vector4Distance(arg_0: C_Vector4, arg_1: C_Vector4) -> f32;
    pub fn Vector4DistanceSqr(arg_0: C_Vector4, arg_1: C_Vector4) -> f32;
    pub fn Vector4Scale(arg_0: C_Vector4, arg_1: f32) -> C_Vector4;
    pub fn Vector4Multiply(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn Vector4Negate(arg_0: C_Vector4) -> C_Vector4;
    pub fn Vector4Divide(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn Vector4Normalize(arg_0: C_Vector4) -> C_Vector4;
    pub fn Vector4Min(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn Vector4Max(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn Vector4Lerp(arg_0: C_Vector4, arg_1: C_Vector4, arg_2: f32) -> C_Vector4;
    pub fn Vector4MoveTowards(arg_0: C_Vector4, arg_1: C_Vector4, arg_2: f32) -> C_Vector4;
    pub fn Vector4Invert(arg_0: C_Vector4) -> C_Vector4;
    pub fn Vector4Equals(arg_0: C_Vector4, arg_1: C_Vector4) -> c_int;
    pub fn MatrixDeterminant(arg_0: C_Matrix) -> f32;
    pub fn MatrixTrace(arg_0: C_Matrix) -> f32;
    pub fn MatrixTranspose(arg_0: C_Matrix) -> C_Matrix;
    pub fn MatrixInvert(arg_0: C_Matrix) -> C_Matrix;
    pub fn MatrixIdentity() -> C_Matrix;
    pub fn MatrixAdd(arg_0: C_Matrix, arg_1: C_Matrix) -> C_Matrix;
    pub fn MatrixSubtract(arg_0: C_Matrix, arg_1: C_Matrix) -> C_Matrix;
    pub fn MatrixMultiply(arg_0: C_Matrix, arg_1: C_Matrix) -> C_Matrix;
    pub fn MatrixMultiplyValue(arg_0: C_Matrix, arg_1: f32) -> C_Matrix;
    pub fn MatrixTranslate(arg_0: f32, arg_1: f32, arg_2: f32) -> C_Matrix;
    pub fn MatrixRotate(arg_0: C_Vector3, arg_1: f32) -> C_Matrix;
    pub fn MatrixRotateX(arg_0: f32) -> C_Matrix;
    pub fn MatrixRotateY(arg_0: f32) -> C_Matrix;
    pub fn MatrixRotateZ(arg_0: f32) -> C_Matrix;
    pub fn MatrixRotateXYZ(arg_0: C_Vector3) -> C_Matrix;
    pub fn MatrixRotateZYX(arg_0: C_Vector3) -> C_Matrix;
    pub fn MatrixScale(arg_0: f32, arg_1: f32, arg_2: f32) -> C_Matrix;
    pub fn MatrixFrustum(arg_0: f64, arg_1: f64, arg_2: f64, arg_3: f64, arg_4: f64, arg_5: f64) -> C_Matrix;
    pub fn MatrixPerspective(arg_0: f64, arg_1: f64, arg_2: f64, arg_3: f64) -> C_Matrix;
    pub fn MatrixOrtho(arg_0: f64, arg_1: f64, arg_2: f64, arg_3: f64, arg_4: f64, arg_5: f64) -> C_Matrix;
    pub fn MatrixLookAt(arg_0: C_Vector3, arg_1: C_Vector3, arg_2: C_Vector3) -> C_Matrix;
    pub fn MatrixToFloatV(arg_0: C_Matrix) -> C_float16;
    pub fn QuaternionAdd(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn QuaternionAddValue(arg_0: C_Vector4, arg_1: f32) -> C_Vector4;
    pub fn QuaternionSubtract(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn QuaternionSubtractValue(arg_0: C_Vector4, arg_1: f32) -> C_Vector4;
    pub fn QuaternionIdentity() -> C_Vector4;
    pub fn QuaternionLength(arg_0: C_Vector4) -> f32;
    pub fn QuaternionNormalize(arg_0: C_Vector4) -> C_Vector4;
    pub fn QuaternionInvert(arg_0: C_Vector4) -> C_Vector4;
    pub fn QuaternionMultiply(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn QuaternionScale(arg_0: C_Vector4, arg_1: f32) -> C_Vector4;
    pub fn QuaternionDivide(arg_0: C_Vector4, arg_1: C_Vector4) -> C_Vector4;
    pub fn QuaternionLerp(arg_0: C_Vector4, arg_1: C_Vector4, arg_2: f32) -> C_Vector4;
    pub fn QuaternionNlerp(arg_0: C_Vector4, arg_1: C_Vector4, arg_2: f32) -> C_Vector4;
    pub fn QuaternionSlerp(arg_0: C_Vector4, arg_1: C_Vector4, arg_2: f32) -> C_Vector4;
    pub fn QuaternionCubicHermiteSpline(arg_0: C_Vector4, arg_1: C_Vector4, arg_2: C_Vector4, arg_3: C_Vector4, arg_4: f32) -> C_Vector4;
    pub fn QuaternionFromVector3ToVector3(arg_0: C_Vector3, arg_1: C_Vector3) -> C_Vector4;
    pub fn QuaternionFromMatrix(arg_0: C_Matrix) -> C_Vector4;
    pub fn QuaternionToMatrix(arg_0: C_Vector4) -> C_Matrix;
    pub fn QuaternionFromAxisAngle(arg_0: C_Vector3, arg_1: f32) -> C_Vector4;
    pub fn QuaternionToAxisAngle(arg_0: C_Vector4, arg_1: *mut C_Vector3, arg_2: *mut f32);
    pub fn QuaternionFromEuler(arg_0: f32, arg_1: f32, arg_2: f32) -> C_Vector4;
    pub fn QuaternionToEuler(arg_0: C_Vector4) -> C_Vector3;
    pub fn QuaternionTransform(arg_0: C_Vector4, arg_1: C_Matrix) -> C_Vector4;
    pub fn QuaternionEquals(arg_0: C_Vector4, arg_1: C_Vector4) -> c_int;
    pub fn MatrixCompose(arg_0: C_Vector3, arg_1: C_Vector4, arg_2: C_Vector3) -> C_Matrix;
    pub fn MatrixDecompose(arg_0: C_Matrix, arg_1: *mut C_Vector3, arg_2: *mut C_Vector4, arg_3: *mut C_Vector3);
}

// ===========================================================================
// Solix C-ABI Native Wrapper Functions
// ===========================================================================
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_InitWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "InitWindow expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    InitWindow((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, str_ptr_2);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CloseWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    CloseWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_WindowShouldClose(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = WindowShouldClose();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowReady(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowReady();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowFullscreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowFullscreen();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowHidden(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowHidden();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowMinimized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowMinimized();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowMaximized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowMaximized();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowFocused(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowFocused();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowResized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsWindowResized();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWindowState(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsWindowState expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsWindowState((*args.add(0)).payload as c_uint);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowState(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowState expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowState((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ClearWindowState(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ClearWindowState expected 1 arguments");
        }
        return SolixCValue::null();
    }
    ClearWindowState((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ToggleFullscreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ToggleFullscreen();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ToggleBorderlessWindowed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ToggleBorderlessWindowed();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MaximizeWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    MaximizeWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MinimizeWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    MinimizeWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_RestoreWindow(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    RestoreWindow();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowIcon(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowIcon expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowIcon(unpack_image(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowIcons(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowIcons expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    SetWindowIcons(&mut image_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowTitle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowTitle expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    SetWindowTitle(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowPosition expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowPosition((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowMonitor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowMonitor expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowMonitor((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowMinSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowMinSize expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowMinSize((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowMaxSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowMaxSize expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowMaxSize((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowSize expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowSize((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowOpacity(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetWindowOpacity expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetWindowOpacity((*args.add(0)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetWindowFocused(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SetWindowFocused();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWindowHandle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWindowHandle();
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetScreenWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenWidth();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetScreenHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetScreenHeight();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRenderWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRenderWidth();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRenderHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetRenderHeight();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMonitorCount();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCurrentMonitor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCurrentMonitor();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorPosition expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorPosition((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorWidth expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorWidth((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorHeight expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorHeight((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorPhysicalWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorPhysicalWidth expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorPhysicalWidth((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorPhysicalHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorPhysicalHeight expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorPhysicalHeight((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorRefreshRate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorRefreshRate expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorRefreshRate((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWindowPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWindowPosition();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWindowScaleDPI(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWindowScaleDPI();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMonitorName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMonitorName expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMonitorName((*args.add(0)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetClipboardText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetClipboardText expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    SetClipboardText(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetClipboardText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetClipboardText();
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetClipboardImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetClipboardImage();
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EnableEventWaiting(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EnableEventWaiting();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DisableEventWaiting(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DisableEventWaiting();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ShowCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    ShowCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_HideCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    HideCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsCursorHidden(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsCursorHidden();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EnableCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EnableCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DisableCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    DisableCursor();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsCursorOnScreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsCursorOnScreen();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ClearBackground(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ClearBackground expected 1 arguments");
        }
        return SolixCValue::null();
    }
    ClearBackground(unpack_color(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginDrawing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    BeginDrawing();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndDrawing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndDrawing();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginMode2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginMode2D expected 1 arguments");
        }
        return SolixCValue::null();
    }
    BeginMode2D(unpack_camera2d(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndMode2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndMode2D();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginMode3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginMode3D expected 1 arguments");
        }
        return SolixCValue::null();
    }
    BeginMode3D(unpack_camera3d(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndMode3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndMode3D();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginTextureMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginTextureMode expected 1 arguments");
        }
        return SolixCValue::null();
    }
    BeginTextureMode(unpack_render_texture(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndTextureMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndTextureMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginShaderMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginShaderMode expected 1 arguments");
        }
        return SolixCValue::null();
    }
    BeginShaderMode(unpack_shader(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndShaderMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndShaderMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginBlendMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginBlendMode expected 1 arguments");
        }
        return SolixCValue::null();
    }
    BeginBlendMode((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndBlendMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndBlendMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginScissorMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginScissorMode expected 4 arguments");
        }
        return SolixCValue::null();
    }
    BeginScissorMode((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndScissorMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndScissorMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_BeginVrStereoMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "BeginVrStereoMode expected 1 arguments");
        }
        return SolixCValue::null();
    }
    BeginVrStereoMode(unpack_vr_stereo_config(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EndVrStereoMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    EndVrStereoMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadVrStereoConfig(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadVrStereoConfig expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadVrStereoConfig(unpack_vr_device_info(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vr_stereo_config(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadVrStereoConfig(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadVrStereoConfig expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadVrStereoConfig(unpack_vr_stereo_config(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadShader expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadShader(str_ptr_0, str_ptr_1);
    SolixCValue::from_ref(pack_shader(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadShaderFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadShaderFromMemory expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadShaderFromMemory(str_ptr_0, str_ptr_1);
    SolixCValue::from_ref(pack_shader(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsShaderValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsShaderValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsShaderValid(unpack_shader(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetShaderLocation(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetShaderLocation expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetShaderLocation(unpack_shader(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetShaderLocationAttrib(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetShaderLocationAttrib expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetShaderLocationAttrib(unpack_shader(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetShaderValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetShaderValue expected 4 arguments");
        }
        return SolixCValue::null();
    }
    SetShaderValue(unpack_shader(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).payload as usize as *mut c_void, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetShaderValueV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetShaderValueV expected 5 arguments");
        }
        return SolixCValue::null();
    }
    SetShaderValueV(unpack_shader(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).payload as usize as *mut c_void, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetShaderValueMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetShaderValueMatrix expected 3 arguments");
        }
        return SolixCValue::null();
    }
    SetShaderValueMatrix(unpack_shader(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, unpack_matrix(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetShaderValueTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetShaderValueTexture expected 3 arguments");
        }
        return SolixCValue::null();
    }
    SetShaderValueTexture(unpack_shader(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, unpack_texture(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadShader expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadShader(unpack_shader(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetScreenToWorldRay(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetScreenToWorldRay expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetScreenToWorldRay(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_camera3d(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_ray(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetScreenToWorldRayEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetScreenToWorldRayEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetScreenToWorldRayEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_camera3d(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_ref(pack_ray(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWorldToScreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetWorldToScreen expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetWorldToScreen(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_camera3d(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWorldToScreenEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetWorldToScreenEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetWorldToScreenEx(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_camera3d(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWorldToScreen2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetWorldToScreen2D expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetWorldToScreen2D(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_camera2d(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetScreenToWorld2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetScreenToWorld2D expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetScreenToWorld2D(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_camera2d(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCameraMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCameraMatrix expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetCameraMatrix(unpack_camera3d(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCameraMatrix2D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCameraMatrix2D expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetCameraMatrix2D(unpack_camera2d(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetTargetFPS(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetTargetFPS expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetTargetFPS((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFrameTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetFrameTime();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTime();
    SolixCValue::from_f64(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFPS(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetFPS();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SwapScreenBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    SwapScreenBuffer();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PollInputEvents(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    PollInputEvents();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_WaitTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "WaitTime expected 1 arguments");
        }
        return SolixCValue::null();
    }
    WaitTime((*args.add(0)).as_f64());
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetRandomSeed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetRandomSeed expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetRandomSeed((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRandomValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetRandomValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetRandomValue((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadRandomSequence(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadRandomSequence expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadRandomSequence((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadRandomSequence(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadRandomSequence expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadRandomSequence((*args.add(0)).payload as usize as *mut c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TakeScreenshot(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TakeScreenshot expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    TakeScreenshot(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetConfigFlags(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetConfigFlags expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetConfigFlags((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_OpenURL(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "OpenURL expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    OpenURL(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetTraceLogLevel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetTraceLogLevel expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetTraceLogLevel((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TraceLog(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TraceLog expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    TraceLog((*args.add(0)).as_i32() as c_int, c"%s".as_ptr(), str_ptr_1);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetTraceLogCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetTraceLogCallback expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetTraceLogCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MemAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MemAlloc expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MemAlloc((*args.add(0)).payload as c_uint);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MemRealloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MemRealloc expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MemRealloc((*args.add(0)).payload as usize as *mut c_void, (*args.add(1)).payload as c_uint);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MemFree(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MemFree expected 1 arguments");
        }
        return SolixCValue::null();
    }
    MemFree((*args.add(0)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFileData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFileData expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadFileData(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadFileData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadFileData expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadFileData((*args.add(0)).payload as usize as *mut u8);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SaveFileData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SaveFileData expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = SaveFileData(str_ptr_0, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportDataAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportDataAsCode expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportDataAsCode(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int, str_ptr_2);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFileText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFileText expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadFileText(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadFileText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadFileText expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    UnloadFileText(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SaveFileText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SaveFileText expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = SaveFileText(str_ptr_0, str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetLoadFileDataCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetLoadFileDataCallback expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetLoadFileDataCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetSaveFileDataCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetSaveFileDataCallback expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetSaveFileDataCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetLoadFileTextCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetLoadFileTextCallback expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetLoadFileTextCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetSaveFileTextCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetSaveFileTextCallback expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetSaveFileTextCallback(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileRename(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileRename expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileRename(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileRemove(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileRemove expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileRemove(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileCopy expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileCopy(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileMove(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileMove expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileMove(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileTextReplace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileTextReplace expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileTextReplace(str_ptr_0, str_ptr_1, str_ptr_2);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileTextFindIndex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileTextFindIndex expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileTextFindIndex(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FileExists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FileExists expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = FileExists(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DirectoryExists(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DirectoryExists expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = DirectoryExists(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsFileExtension(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsFileExtension expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = IsFileExtension(str_ptr_0, str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsFileHidden(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsFileHidden expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = IsFileHidden(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFileLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetFileLength expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetFileLength(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFileModTime(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetFileModTime expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetFileModTime(str_ptr_0);
    SolixCValue::from_i64(call_result as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFileExtension(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetFileExtension expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetFileExtension(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFileName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetFileName expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetFileName(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFileNameWithoutExt(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetFileNameWithoutExt expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetFileNameWithoutExt(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetDirectoryPath(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetDirectoryPath expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetDirectoryPath(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetPrevDirectoryPath(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetPrevDirectoryPath expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetPrevDirectoryPath(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetWorkingDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetWorkingDirectory();
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetApplicationDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetApplicationDirectory();
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MakeDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MakeDirectory expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = MakeDirectory(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ChangeDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ChangeDirectory expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ChangeDirectory(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsPathFile(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsPathFile expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = IsPathFile(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsPathDirectory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsPathDirectory expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = IsPathDirectory(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsPathAbsolute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsPathAbsolute expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = IsPathAbsolute(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsFileNameValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsFileNameValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = IsFileNameValid(str_ptr_0);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadDirectoryFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadDirectoryFiles expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadDirectoryFiles(str_ptr_0);
    SolixCValue::from_ref(pack_file_path_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadDirectoryFilesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadDirectoryFilesEx expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadDirectoryFilesEx(str_ptr_0, str_ptr_1, (*args.add(2)).as_bool());
    SolixCValue::from_ref(pack_file_path_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadDirectoryFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadDirectoryFiles expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadDirectoryFiles(unpack_file_path_list(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsFileDropped(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsFileDropped();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadDroppedFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadDroppedFiles();
    SolixCValue::from_ref(pack_file_path_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadDroppedFiles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadDroppedFiles expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadDroppedFiles(unpack_file_path_list(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetDirectoryFileCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetDirectoryFileCount expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetDirectoryFileCount(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetDirectoryFileCountEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetDirectoryFileCountEx expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetDirectoryFileCountEx(str_ptr_0, str_ptr_1, (*args.add(2)).as_bool());
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CompressData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CompressData expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = CompressData(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DecompressData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DecompressData expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = DecompressData(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_EncodeDataBase64(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "EncodeDataBase64 expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = EncodeDataBase64(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DecodeDataBase64(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DecodeDataBase64 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = DecodeDataBase64(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ComputeCRC32(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ComputeCRC32 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let call_result = ComputeCRC32(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ComputeMD5(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ComputeMD5 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let call_result = ComputeMD5(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ComputeSHA1(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ComputeSHA1 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let call_result = ComputeSHA1(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ComputeSHA256(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ComputeSHA256 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let call_result = ComputeSHA256(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadAutomationEventList expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadAutomationEventList(str_ptr_0);
    SolixCValue::from_ref(pack_automation_event_list(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadAutomationEventList expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadAutomationEventList(unpack_automation_event_list(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportAutomationEventList expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportAutomationEventList(unpack_automation_event_list(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAutomationEventList(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAutomationEventList expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let automation_event_list_addr_0 = (*args.add(0)).as_ref();
    let mut automation_event_list_mut_0 = unpack_automation_event_list(vm_context, automation_event_list_addr_0);
    SetAutomationEventList(&mut automation_event_list_mut_0);
    writeback_automation_event_list(vm_context, automation_event_list_addr_0, &automation_event_list_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAutomationEventBaseFrame(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAutomationEventBaseFrame expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetAutomationEventBaseFrame((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_StartAutomationEventRecording(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StartAutomationEventRecording();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_StopAutomationEventRecording(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    StopAutomationEventRecording();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PlayAutomationEvent(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PlayAutomationEvent expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PlayAutomationEvent(unpack_automation_event(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsKeyPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsKeyPressed expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsKeyPressed((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsKeyPressedRepeat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsKeyPressedRepeat expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsKeyPressedRepeat((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsKeyDown(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsKeyDown expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsKeyDown((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsKeyReleased(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsKeyReleased expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsKeyReleased((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsKeyUp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsKeyUp expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsKeyUp((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetKeyPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetKeyPressed();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCharPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetCharPressed();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetKeyName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetKeyName expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetKeyName((*args.add(0)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetExitKey(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetExitKey expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetExitKey((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsGamepadAvailable(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsGamepadAvailable expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsGamepadAvailable((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGamepadName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetGamepadName expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetGamepadName((*args.add(0)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsGamepadButtonPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsGamepadButtonPressed expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsGamepadButtonPressed((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsGamepadButtonDown(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsGamepadButtonDown expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsGamepadButtonDown((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsGamepadButtonReleased(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsGamepadButtonReleased expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsGamepadButtonReleased((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsGamepadButtonUp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsGamepadButtonUp expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsGamepadButtonUp((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGamepadButtonPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGamepadButtonPressed();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGamepadAxisCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetGamepadAxisCount expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetGamepadAxisCount((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGamepadAxisMovement(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetGamepadAxisMovement expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetGamepadAxisMovement((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetGamepadMappings(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetGamepadMappings expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = SetGamepadMappings(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetGamepadVibration(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetGamepadVibration expected 4 arguments");
        }
        return SolixCValue::null();
    }
    SetGamepadVibration((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMouseButtonPressed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMouseButtonPressed expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMouseButtonPressed((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMouseButtonDown(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMouseButtonDown expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMouseButtonDown((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMouseButtonReleased(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMouseButtonReleased expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMouseButtonReleased((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMouseButtonUp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMouseButtonUp expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMouseButtonUp((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMouseX(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseX();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMouseY(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseY();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMousePosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMousePosition();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMouseDelta(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseDelta();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMousePosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMousePosition expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetMousePosition((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMouseOffset(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMouseOffset expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetMouseOffset((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMouseScale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMouseScale expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetMouseScale((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMouseWheelMove(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseWheelMove();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMouseWheelMoveV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMouseWheelMoveV();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMouseCursor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMouseCursor expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetMouseCursor((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTouchX(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchX();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTouchY(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchY();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTouchPosition(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetTouchPosition expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetTouchPosition((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTouchPointId(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetTouchPointId expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetTouchPointId((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTouchPointCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetTouchPointCount();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetGesturesEnabled(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetGesturesEnabled expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetGesturesEnabled((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsGestureDetected(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsGestureDetected expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsGestureDetected((*args.add(0)).payload as c_uint);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGestureDetected(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureDetected();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGestureHoldDuration(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureHoldDuration();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGestureDragVector(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureDragVector();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGestureDragAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGestureDragAngle();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGesturePinchVector(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGesturePinchVector();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGesturePinchAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetGesturePinchAngle();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateCamera(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateCamera expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let camera3d_addr_0 = (*args.add(0)).as_ref();
    let mut camera3d_mut_0 = unpack_camera3d(vm_context, camera3d_addr_0);
    UpdateCamera(&mut camera3d_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_camera3d(vm_context, camera3d_addr_0, &camera3d_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateCameraPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateCameraPro expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let camera3d_addr_0 = (*args.add(0)).as_ref();
    let mut camera3d_mut_0 = unpack_camera3d(vm_context, camera3d_addr_0);
    UpdateCameraPro(&mut camera3d_mut_0, unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32);
    writeback_camera3d(vm_context, camera3d_addr_0, &camera3d_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetShapesTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetShapesTexture expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetShapesTexture(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetShapesTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetShapesTexture();
    SolixCValue::from_ref(pack_texture(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetShapesTextureRectangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetShapesTextureRectangle();
    SolixCValue::from_ref(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPixel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPixel expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawPixel((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPixelV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPixelV expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawPixelV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLine expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawLine((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLineV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLineV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawLineV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLineEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLineEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawLineEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLineStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLineStrip expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawLineStrip(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLineBezier(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLineBezier expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawLineBezier(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLineDashed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLineDashed expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawLineDashed(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangle expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawTriangle(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangleGradient expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawTriangleGradient(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangleLines expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawTriangleLines(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangleLinesEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawTriangleLinesEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangleFan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangleFan expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawTriangleFan(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangleStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangleStrip expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawTriangleStrip(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangle expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangle((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleRec expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleRec(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectanglePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectanglePro expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectanglePro(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleGradientV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleGradientV expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleGradientV((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleGradientH(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleGradientH expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleGradientH((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleGradientEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleGradientEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleGradientEx(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleLines expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleLines((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleLinesEx expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleLinesEx(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleRounded(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleRounded expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleRounded(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleRoundedLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleRoundedLines expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleRoundedLines(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRectangleRoundedLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRectangleRoundedLinesEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawRectangleRoundedLinesEx(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPoly(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPoly expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawPoly(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPolyLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPolyLines expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawPolyLines(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPolyLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPolyLinesEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawPolyLinesEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircle expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircle((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleGradient expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleGradient(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_color(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleSector(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleSector expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleSector(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleSectorLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleSectorLines expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleSectorLines(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleSectorLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleSectorLinesEx expected 7 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleSectorLinesEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_f64() as f32, unpack_color(vm_context, (*args.add(6)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleLines expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleLines((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleLinesV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleLinesV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleLinesV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircleLinesEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircleLinesEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawEllipse(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawEllipse expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawEllipse((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawEllipseV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawEllipseV expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawEllipseV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawEllipseLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawEllipseLines expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawEllipseLines((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawEllipseLinesV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawEllipseLinesV expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawEllipseLinesV(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawEllipseLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawEllipseLinesEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawEllipseLinesEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRing expected 7 arguments");
        }
        return SolixCValue::null();
    }
    DrawRing(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32, (*args.add(5)).as_i32() as c_int, unpack_color(vm_context, (*args.add(6)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRingLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRingLines expected 7 arguments");
        }
        return SolixCValue::null();
    }
    DrawRingLines(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32, (*args.add(5)).as_i32() as c_int, unpack_color(vm_context, (*args.add(6)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRingLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 8 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRingLinesEx expected 8 arguments");
        }
        return SolixCValue::null();
    }
    DrawRingLinesEx(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32, (*args.add(5)).as_i32() as c_int, (*args.add(6)).as_f64() as f32, unpack_color(vm_context, (*args.add(7)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineLinear expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawSplineLinear(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineBasis(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineBasis expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawSplineBasis(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineCatmullRom(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineCatmullRom expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawSplineCatmullRom(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineBezierQuadratic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineBezierQuadratic expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawSplineBezierQuadratic(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineBezierCubic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineBezierCubic expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector2> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    DrawSplineBezierCubic(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineSegmentLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineSegmentLinear expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawSplineSegmentLinear(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineSegmentBasis(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineSegmentBasis expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawSplineSegmentBasis(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineSegmentCatmullRom(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineSegmentCatmullRom expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawSplineSegmentCatmullRom(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineSegmentBezierQuadratic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineSegmentBezierQuadratic expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawSplineSegmentBezierQuadratic(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSplineSegmentBezierCubic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSplineSegmentBezierCubic expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawSplineSegmentBezierCubic(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetSplinePointLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetSplinePointLinear expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetSplinePointLinear(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetSplinePointBasis(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetSplinePointBasis expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetSplinePointBasis(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetSplinePointCatmullRom(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetSplinePointCatmullRom expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetSplinePointCatmullRom(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetSplinePointBezierQuadratic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetSplinePointBezierQuadratic expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetSplinePointBezierQuadratic(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetSplinePointBezierCubic(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetSplinePointBezierCubic expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetSplinePointBezierCubic(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionRecs(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionRecs expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionRecs(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionCircles(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionCircles expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionCircles(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionCircleRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionCircleRec expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionCircleRec(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_rectangle(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionCircleLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionCircleLine expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionCircleLine(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionPointRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionPointRec expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionPointRec(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionPointCircle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionPointCircle expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionPointCircle(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionPointTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionPointTriangle expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionPointTriangle(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionPointLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionPointLine expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionPointLine(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionPointPoly(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionPointPoly expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut struct_vec_1: Vec<C_Vector2> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            struct_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_1, elem_idx) as u32;
                struct_vec_1.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    let call_result = CheckCollisionPointPoly(unpack_vector2(vm_context, (*args.add(0)).as_ref()), if arr_addr_1 > 0 { struct_vec_1.as_ptr() } else { std::ptr::null() }, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionLines expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let vector2_addr_4 = (*args.add(4)).as_ref();
    let mut vector2_mut_4 = unpack_vector2(vm_context, vector2_addr_4);
    let call_result = CheckCollisionLines(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), &mut vector2_mut_4);
    writeback_vector2(vm_context, vector2_addr_4, &vector2_mut_4);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCollisionRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCollisionRec expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetCollisionRec(unpack_rectangle(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImage expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadImage(str_ptr_0);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageRaw(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImageRaw expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadImageRaw(str_ptr_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageAnim(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImageAnim expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadImageAnim(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageAnimFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImageAnimFromMemory expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<u8> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as u8);
            }
        }
    }
    let arr_addr_3 = (*args.add(3)).as_ref();
    let mut prim_vec_3: Vec<c_int> = Vec::new();
    if arr_addr_3 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_3);
            prim_vec_3.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_3, elem_idx);
                prim_vec_3.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadImageAnimFromMemory(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int, if arr_addr_3 > 0 { prim_vec_3.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_3 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_3);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_3[elem_idx as usize];
                ctx.array_set_u64(arr_addr_3, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImageFromMemory expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<u8> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as u8);
            }
        }
    }
    let call_result = LoadImageFromMemory(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageFromTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImageFromTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadImageFromTexture(unpack_texture(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageFromScreen(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadImageFromScreen();
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsImageValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsImageValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsImageValid(unpack_image(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadImage expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadImage(unpack_image(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportImage expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportImage(unpack_image(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportImageToMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportImageToMemory expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = ExportImageToMemory(unpack_image(vm_context, (*args.add(0)).as_ref()), str_ptr_1, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportImageAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportImageAsCode expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportImageAsCode(unpack_image(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageColor expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageColor((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageGradientLinear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageGradientLinear expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageGradientLinear((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageGradientRadial(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageGradientRadial expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageGradientRadial((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageGradientSquare(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageGradientSquare expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageGradientSquare((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageChecked(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageChecked expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageChecked((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageWhiteNoise(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageWhiteNoise expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageWhiteNoise((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImagePerlinNoise(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImagePerlinNoise expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImagePerlinNoise((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageCellular(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageCellular expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenImageCellular((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageText expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GenImageText((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, str_ptr_2);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageCopy expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ImageCopy(unpack_image(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageFromImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageFromImage expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ImageFromImage(unpack_image(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageFromChannel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageFromChannel expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ImageFromChannel(unpack_image(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageText expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ImageText(str_ptr_0, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageTextEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ImageTextEx(unpack_font(vm_context, (*args.add(0)).as_ref()), str_ptr_1, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageFormat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageFormat expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageFormat(&mut image_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageToPOT(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageToPOT expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageToPOT(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageCrop(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageCrop expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageCrop(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageAlphaCrop(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageAlphaCrop expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaCrop(&mut image_mut_0, (*args.add(1)).as_f64() as f32);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageAlphaClear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageAlphaClear expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaClear(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageAlphaMask(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageAlphaMask expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaMask(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageAlphaPremultiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageAlphaPremultiply expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageAlphaPremultiply(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageBlurGaussian(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageBlurGaussian expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageBlurGaussian(&mut image_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageKernelConvolution(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageKernelConvolution expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<f32> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 });
            }
        }
    }
    ImageKernelConvolution(&mut image_mut_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageResize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageResize expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageResize(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageResizeNN(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageResizeNN expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageResizeNN(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageResizeCanvas(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageResizeCanvas expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageResizeCanvas(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageMipmaps(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageMipmaps expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageMipmaps(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDither(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDither expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDither(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageFlipVertical(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageFlipVertical expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageFlipVertical(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageFlipHorizontal(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageFlipHorizontal expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageFlipHorizontal(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageRotate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageRotate expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageRotate(&mut image_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageRotateCW(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageRotateCW expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageRotateCW(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageRotateCCW(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageRotateCCW expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageRotateCCW(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageColorTint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageColorTint expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorTint(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageColorInvert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageColorInvert expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorInvert(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageColorGrayscale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageColorGrayscale expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorGrayscale(&mut image_mut_0);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageColorContrast(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageColorContrast expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorContrast(&mut image_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageColorBrightness(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageColorBrightness expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorBrightness(&mut image_mut_0, (*args.add(1)).as_i32() as c_int);
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageColorReplace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageColorReplace expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageColorReplace(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImageColors(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImageColors expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadImageColors(unpack_image(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadImagePalette(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadImagePalette expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadImagePalette(unpack_image(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadImageColors(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadImageColors expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadImageColors((*args.add(0)).payload as usize as *mut C_Color);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadImagePalette(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadImagePalette expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadImagePalette((*args.add(0)).payload as usize as *mut C_Color);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetImageAlphaBorder(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetImageAlphaBorder expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetImageAlphaBorder(unpack_image(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetImageColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetImageColor expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetImageColor(unpack_image(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageClearBackground(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageClearBackground expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageClearBackground(&mut image_mut_0, unpack_color(vm_context, (*args.add(1)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawPixel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawPixel expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawPixel(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawPixelV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawPixelV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawPixelV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawLine(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawLine expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLine(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawLineV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawLineV expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLineV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawLineEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawLineEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawLineEx(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawLineStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawLineStrip expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut struct_vec_1: Vec<C_Vector2> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            struct_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_1, elem_idx) as u32;
                struct_vec_1.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    ImageDrawLineStrip(&mut image_mut_0, if arr_addr_1 > 0 { struct_vec_1.as_ptr() } else { std::ptr::null() }, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTriangle expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangle(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTriangleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTriangleGradient expected 7 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangleGradient(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()), unpack_color(vm_context, (*args.add(6)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTriangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTriangleLines expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawTriangleLines(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTriangleFan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTriangleFan expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut struct_vec_1: Vec<C_Vector2> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            struct_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_1, elem_idx) as u32;
                struct_vec_1.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    ImageDrawTriangleFan(&mut image_mut_0, if arr_addr_1 > 0 { struct_vec_1.as_ptr() } else { std::ptr::null() }, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTriangleStrip(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTriangleStrip expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut struct_vec_1: Vec<C_Vector2> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            struct_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_1, elem_idx) as u32;
                struct_vec_1.push(unpack_vector2(vm_context, child_addr));
            }
        }
    }
    ImageDrawTriangleStrip(&mut image_mut_0, if arr_addr_1 > 0 { struct_vec_1.as_ptr() } else { std::ptr::null() }, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectangle expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangle(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectangleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectangleV expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectangleRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectangleRec expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleRec(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectanglePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectanglePro expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectanglePro(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectangleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectangleLines expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleLines(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectangleLinesEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectangleLinesEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleLinesEx(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawRectangleGradientEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawRectangleGradientEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawRectangleGradientEx(&mut image_mut_0, unpack_rectangle(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawCircle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawCircle expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircle(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawCircleV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawCircleV expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawCircleLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawCircleLines expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleLines(&mut image_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawCircleLinesV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawCircleLinesV expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleLinesV(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawCircleGradient(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawCircleGradient expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawCircleGradient(&mut image_mut_0, unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawImage expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImage(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawImageEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawImageEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImageEx(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawImageRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawImageRec expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImageRec(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).as_ref()), unpack_rectangle(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), unpack_color(vm_context, (*args.add(4)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawImagePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawImagePro expected 7 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    ImageDrawImagePro(&mut image_mut_0, unpack_image(vm_context, (*args.add(1)).as_ref()), unpack_rectangle(vm_context, (*args.add(2)).as_ref()), unpack_rectangle(vm_context, (*args.add(3)).as_ref()), unpack_vector2(vm_context, (*args.add(4)).as_ref()), (*args.add(5)).as_f64() as f32, unpack_color(vm_context, (*args.add(6)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawText expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    ImageDrawText(&mut image_mut_0, str_ptr_1, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTextEx expected 7 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    ImageDrawTextEx(&mut image_mut_0, unpack_font(vm_context, (*args.add(1)).as_ref()), str_ptr_2, unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, (*args.add(5)).as_f64() as f32, unpack_color(vm_context, (*args.add(6)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ImageDrawTextPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 9 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ImageDrawTextPro expected 9 arguments");
        }
        return SolixCValue::null();
    }
    let image_addr_0 = (*args.add(0)).as_ref();
    let mut image_mut_0 = unpack_image(vm_context, image_addr_0);
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    ImageDrawTextPro(&mut image_mut_0, unpack_font(vm_context, (*args.add(1)).as_ref()), str_ptr_2, unpack_vector2(vm_context, (*args.add(3)).as_ref()), unpack_vector2(vm_context, (*args.add(4)).as_ref()), (*args.add(5)).as_f64() as f32, (*args.add(6)).as_f64() as f32, (*args.add(7)).as_f64() as f32, unpack_color(vm_context, (*args.add(8)).as_ref()));
    writeback_image(vm_context, image_addr_0, &image_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadTexture(str_ptr_0);
    SolixCValue::from_ref(pack_texture(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadTextureFromImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadTextureFromImage expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadTextureFromImage(unpack_image(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_texture(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadTextureCubemap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadTextureCubemap expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadTextureCubemap(unpack_image(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_texture(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadRenderTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadRenderTexture expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadRenderTexture((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_render_texture(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadRenderTextureEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadRenderTextureEx expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadRenderTextureEx((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_render_texture(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsTextureValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsTextureValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsTextureValid(unpack_texture(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadTexture(unpack_texture(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsRenderTextureValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsRenderTextureValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsRenderTextureValid(unpack_render_texture(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadRenderTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadRenderTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadRenderTexture(unpack_render_texture(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateTexture expected 2 arguments");
        }
        return SolixCValue::null();
    }
    UpdateTexture(unpack_texture(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateTextureRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateTextureRec expected 3 arguments");
        }
        return SolixCValue::null();
    }
    UpdateTextureRec(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenTextureMipmaps(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenTextureMipmaps expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let texture_addr_0 = (*args.add(0)).as_ref();
    let mut texture_mut_0 = unpack_texture(vm_context, texture_addr_0);
    GenTextureMipmaps(&mut texture_mut_0);
    writeback_texture(vm_context, texture_addr_0, &texture_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetTextureFilter(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetTextureFilter expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetTextureFilter(unpack_texture(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetTextureWrap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetTextureWrap expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetTextureWrap(unpack_texture(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTexture expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawTexture(unpack_texture(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextureV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextureV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawTextureV(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextureEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextureEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawTextureEx(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextureRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextureRec expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawTextureRec(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTexturePro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTexturePro expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawTexturePro(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_rectangle(vm_context, (*args.add(1)).as_ref()), unpack_rectangle(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextureNPatch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextureNPatch expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawTextureNPatch(unpack_texture(vm_context, (*args.add(0)).as_ref()), unpack_n_patch_info(vm_context, (*args.add(1)).as_ref()), unpack_rectangle(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorIsEqual(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorIsEqual expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorIsEqual(unpack_color(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Fade(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Fade expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Fade(unpack_color(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorToInt(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorToInt expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorToInt(unpack_color(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorNormalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorNormalize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorNormalize(unpack_color(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorFromNormalized(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorFromNormalized expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorFromNormalized(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorToHSV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorToHSV expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorToHSV(unpack_color(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorFromHSV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorFromHSV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorFromHSV((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorTint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorTint expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorTint(unpack_color(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorBrightness(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorBrightness expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorBrightness(unpack_color(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorContrast(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorContrast expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorContrast(unpack_color(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorAlpha(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorAlpha expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorAlpha(unpack_color(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorAlphaBlend(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorAlphaBlend expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorAlphaBlend(unpack_color(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ColorLerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ColorLerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = ColorLerp(unpack_color(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetColor expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetColor((*args.add(0)).payload as c_uint);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetPixelColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetPixelColor expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetPixelColor((*args.add(0)).payload as usize as *mut c_void, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_color(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetPixelColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetPixelColor expected 3 arguments");
        }
        return SolixCValue::null();
    }
    SetPixelColor((*args.add(0)).payload as usize as *mut c_void, unpack_color(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetPixelDataSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetPixelDataSize expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetPixelDataSize((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetFontDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetFontDefault();
    SolixCValue::from_ref(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFont(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFont expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadFont(str_ptr_0);
    SolixCValue::from_ref(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFontEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFontEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadFontEx(str_ptr_0, (*args.add(1)).as_i32() as c_int, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_ref(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFontFromImage(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFontFromImage expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadFontFromImage(unpack_image(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFontFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFontFromMemory expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<u8> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as u8);
            }
        }
    }
    let arr_addr_4 = (*args.add(4)).as_ref();
    let mut prim_vec_4: Vec<c_int> = Vec::new();
    if arr_addr_4 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_4);
            prim_vec_4.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_4, elem_idx);
                prim_vec_4.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadFontFromMemory(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, if arr_addr_4 > 0 { prim_vec_4.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(5)).as_i32() as c_int);
    SolixCValue::from_ref(pack_font(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsFontValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsFontValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsFontValid(unpack_font(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadFontData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadFontData expected 7 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<u8> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as u8);
            }
        }
    }
    let arr_addr_3 = (*args.add(3)).as_ref();
    let mut prim_vec_3: Vec<c_int> = Vec::new();
    if arr_addr_3 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_3);
            prim_vec_3.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_3, elem_idx);
                prim_vec_3.push(raw_val as c_int);
            }
        }
    }
    let arr_addr_6 = (*args.add(6)).as_ref();
    let mut prim_vec_6: Vec<c_int> = Vec::new();
    if arr_addr_6 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_6);
            prim_vec_6.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_6, elem_idx);
                prim_vec_6.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadFontData(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, if arr_addr_3 > 0 { prim_vec_3.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_i32() as c_int, if arr_addr_6 > 0 { prim_vec_6.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_6 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_6);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_6[elem_idx as usize];
                ctx.array_set_u64(arr_addr_6, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenImageFontAtlas(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenImageFontAtlas expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_GlyphInfo> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_glyph_info(vm_context, child_addr));
            }
        }
    }
    let call_result = GenImageFontAtlas(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_i32() as c_int);
    SolixCValue::from_ref(pack_image(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadFontData(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadFontData expected 2 arguments");
        }
        return SolixCValue::null();
    }
    UnloadFontData((*args.add(0)).payload as usize as *mut C_GlyphInfo, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadFont(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadFont expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadFont(unpack_font(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportFontAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportFontAsCode expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportFontAsCode(unpack_font(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawFPS(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawFPS expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawFPS((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawText expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    DrawText(str_ptr_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    DrawTextEx(unpack_font(vm_context, (*args.add(0)).as_ref()), str_ptr_1, unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 8 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextPro expected 8 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    DrawTextPro(unpack_font(vm_context, (*args.add(0)).as_ref()), str_ptr_1, unpack_vector2(vm_context, (*args.add(2)).as_ref()), unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, (*args.add(5)).as_f64() as f32, (*args.add(6)).as_f64() as f32, unpack_color(vm_context, (*args.add(7)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextCodepoint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextCodepoint expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawTextCodepoint(unpack_font(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, unpack_vector2(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTextCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTextCodepoints expected 7 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    DrawTextCodepoints(unpack_font(vm_context, (*args.add(0)).as_ref()), if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int, unpack_vector2(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, (*args.add(5)).as_f64() as f32, unpack_color(vm_context, (*args.add(6)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetTextLineSpacing(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetTextLineSpacing expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetTextLineSpacing((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MeasureText(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MeasureText expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = MeasureText(str_ptr_0, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MeasureTextEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MeasureTextEx expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = MeasureTextEx(unpack_font(vm_context, (*args.add(0)).as_ref()), str_ptr_1, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MeasureTextCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MeasureTextCodepoints expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = MeasureTextCodepoints(unpack_font(vm_context, (*args.add(0)).as_ref()), if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGlyphIndex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetGlyphIndex expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetGlyphIndex(unpack_font(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGlyphInfo(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetGlyphInfo expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetGlyphInfo(unpack_font(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_glyph_info(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetGlyphAtlasRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetGlyphAtlasRec expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetGlyphAtlasRec(unpack_font(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_rectangle(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadUTF8(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadUTF8 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<c_int> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadUTF8(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(1)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadUTF8(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadUTF8 expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    UnloadUTF8(str_ptr_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadCodepoints expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadCodepoints(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadCodepoints(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadCodepoints expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadCodepoints((*args.add(0)).payload as usize as *mut c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCodepointCount(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCodepointCount expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetCodepointCount(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCodepoint(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCodepoint expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = GetCodepoint(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCodepointNext(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCodepointNext expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = GetCodepointNext(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetCodepointPrevious(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetCodepointPrevious expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = GetCodepointPrevious(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CodepointToUTF8(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CodepointToUTF8 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = CodepointToUTF8((*args.add(0)).as_i32() as c_int, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadTextLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadTextLines expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadTextLines(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadTextLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadTextLines expected 2 arguments");
        }
        return SolixCValue::null();
    }
    UnloadTextLines((*args.add(0)).payload as usize as *mut *mut c_char, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextCopy expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextCopy(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextIsEqual(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextIsEqual expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextIsEqual(str_ptr_0, str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextLength expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextLength(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextFormat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextFormat expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextFormat(c"%s".as_ptr(), str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextSubtext(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextSubtext expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextSubtext(str_ptr_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextRemoveSpaces(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextRemoveSpaces expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextRemoveSpaces(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetTextBetween(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetTextBetween expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = GetTextBetween(str_ptr_0, str_ptr_1, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextReplace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextReplace expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextReplace(str_ptr_0, str_ptr_1, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextReplaceAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextReplaceAlloc expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextReplaceAlloc(str_ptr_0, str_ptr_1, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextReplaceBetween(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextReplaceBetween expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_3 = (*args.add(3)).as_ref();
    let str_keepalive_3: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_3).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_3 = str_keepalive_3.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextReplaceBetween(str_ptr_0, str_ptr_1, str_ptr_2, str_ptr_3);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextReplaceBetweenAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextReplaceBetweenAlloc expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_3 = (*args.add(3)).as_ref();
    let str_keepalive_3: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_3).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_3 = str_keepalive_3.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextReplaceBetweenAlloc(str_ptr_0, str_ptr_1, str_ptr_2, str_ptr_3);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextInsert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextInsert expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextInsert(str_ptr_0, str_ptr_1, (*args.add(2)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextInsertAlloc(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextInsertAlloc expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextInsertAlloc(str_ptr_0, str_ptr_1, (*args.add(2)).as_i32() as c_int);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextJoin(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextJoin expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_2 = (*args.add(2)).as_ref();
    let str_keepalive_2: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_2).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_2 = str_keepalive_2.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextJoin((*args.add(0)).payload as usize as *mut *mut c_char, (*args.add(1)).as_i32() as c_int, str_ptr_2);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextSplit(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextSplit expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    let call_result = TextSplit(str_ptr_0, (*args.add(1)).as_i32() as c_char, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextAppend(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextAppend expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_int> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_int);
            }
        }
    }
    TextAppend(str_ptr_0, str_ptr_1, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextFindIndex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextFindIndex expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextFindIndex(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToUpper(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToUpper expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToUpper(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToLower(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToLower expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToLower(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToPascal(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToPascal expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToPascal(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToSnake(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToSnake expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToSnake(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToCamel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToCamel expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToCamel(str_ptr_0);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToInteger(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToInteger expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToInteger(str_ptr_0);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_TextToFloat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "TextToFloat expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = TextToFloat(str_ptr_0);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawLine3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawLine3D expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawLine3D(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPoint3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPoint3D expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawPoint3D(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCircle3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCircle3D expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawCircle3D(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_vector3(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangle3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangle3D expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawTriangle3D(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawTriangleStrip3D(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawTriangleStrip3D expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut struct_vec_0: Vec<C_Vector3> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            struct_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_0, elem_idx) as u32;
                struct_vec_0.push(unpack_vector3(vm_context, child_addr));
            }
        }
    }
    DrawTriangleStrip3D(if arr_addr_0 > 0 { struct_vec_0.as_ptr() } else { std::ptr::null() }, (*args.add(1)).as_i32() as c_int, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCube(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCube expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawCube(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCubeV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCubeV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawCubeV(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCubeWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCubeWires expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawCubeWires(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCubeWiresV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCubeWiresV expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawCubeWiresV(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSphere expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawSphere(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSphereEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSphereEx expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawSphereEx(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawSphereWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawSphereWires expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawSphereWires(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCylinder(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCylinder expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCylinder(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCylinderEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCylinderEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCylinderEx(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCylinderWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCylinderWires expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCylinderWires(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCylinderWiresEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCylinderWiresEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCylinderWiresEx(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCapsule(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCapsule expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCapsule(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawCapsuleWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawCapsuleWires expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawCapsuleWires(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawPlane(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawPlane expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawPlane(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_color(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawRay(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawRay expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawRay(unpack_ray(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawGrid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawGrid expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawGrid((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadModel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadModel expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadModel(str_ptr_0);
    SolixCValue::from_ref(pack_model(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadModelFromMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadModelFromMesh expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadModelFromMesh(unpack_mesh(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_model(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsModelValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsModelValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsModelValid(unpack_model(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadModel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadModel expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadModel(unpack_model(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetModelBoundingBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetModelBoundingBox expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetModelBoundingBox(unpack_model(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_bounding_box(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawModel(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawModel expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawModel(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawModelEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawModelEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawModelEx(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_vector3(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawModelWires(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawModelWires expected 4 arguments");
        }
        return SolixCValue::null();
    }
    DrawModelWires(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_color(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawModelWiresEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawModelWiresEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawModelWiresEx(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_vector3(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawBoundingBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawBoundingBox expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DrawBoundingBox(unpack_bounding_box(vm_context, (*args.add(0)).as_ref()), unpack_color(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawBillboard(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawBillboard expected 5 arguments");
        }
        return SolixCValue::null();
    }
    DrawBillboard(unpack_camera3d(vm_context, (*args.add(0)).as_ref()), unpack_texture(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32, unpack_color(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawBillboardRec(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawBillboardRec expected 6 arguments");
        }
        return SolixCValue::null();
    }
    DrawBillboardRec(unpack_camera3d(vm_context, (*args.add(0)).as_ref()), unpack_texture(vm_context, (*args.add(1)).as_ref()), unpack_rectangle(vm_context, (*args.add(2)).as_ref()), unpack_vector3(vm_context, (*args.add(3)).as_ref()), unpack_vector2(vm_context, (*args.add(4)).as_ref()), unpack_color(vm_context, (*args.add(5)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawBillboardPro(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 9 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawBillboardPro expected 9 arguments");
        }
        return SolixCValue::null();
    }
    DrawBillboardPro(unpack_camera3d(vm_context, (*args.add(0)).as_ref()), unpack_texture(vm_context, (*args.add(1)).as_ref()), unpack_rectangle(vm_context, (*args.add(2)).as_ref()), unpack_vector3(vm_context, (*args.add(3)).as_ref()), unpack_vector3(vm_context, (*args.add(4)).as_ref()), unpack_vector2(vm_context, (*args.add(5)).as_ref()), unpack_vector2(vm_context, (*args.add(6)).as_ref()), (*args.add(7)).as_f64() as f32, unpack_color(vm_context, (*args.add(8)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UploadMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UploadMesh expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let mesh_addr_0 = (*args.add(0)).as_ref();
    let mut mesh_mut_0 = unpack_mesh(vm_context, mesh_addr_0);
    UploadMesh(&mut mesh_mut_0, (*args.add(1)).as_bool());
    writeback_mesh(vm_context, mesh_addr_0, &mesh_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateMeshBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateMeshBuffer expected 5 arguments");
        }
        return SolixCValue::null();
    }
    UpdateMeshBuffer(unpack_mesh(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_i32() as c_int, (*args.add(2)).payload as usize as *mut c_void, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadMesh expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadMesh(unpack_mesh(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawMesh expected 3 arguments");
        }
        return SolixCValue::null();
    }
    DrawMesh(unpack_mesh(vm_context, (*args.add(0)).as_ref()), unpack_material(vm_context, (*args.add(1)).as_ref()), unpack_matrix(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DrawMeshInstanced(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DrawMeshInstanced expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut struct_vec_2: Vec<C_Matrix> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            struct_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_2, elem_idx) as u32;
                struct_vec_2.push(unpack_matrix(vm_context, child_addr));
            }
        }
    }
    DrawMeshInstanced(unpack_mesh(vm_context, (*args.add(0)).as_ref()), unpack_material(vm_context, (*args.add(1)).as_ref()), if arr_addr_2 > 0 { struct_vec_2.as_ptr() } else { std::ptr::null() }, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMeshBoundingBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMeshBoundingBox expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMeshBoundingBox(unpack_mesh(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_bounding_box(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshTangents(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshTangents expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let mesh_addr_0 = (*args.add(0)).as_ref();
    let mut mesh_mut_0 = unpack_mesh(vm_context, mesh_addr_0);
    GenMeshTangents(&mut mesh_mut_0);
    writeback_mesh(vm_context, mesh_addr_0, &mesh_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportMesh expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportMesh(unpack_mesh(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportMeshAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportMeshAsCode expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportMeshAsCode(unpack_mesh(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshPoly(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshPoly expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshPoly((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshPlane(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshPlane expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshPlane((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshCube(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshCube expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshCube((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshSphere expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshSphere((*args.add(0)).as_f64() as f32, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshHemiSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshHemiSphere expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshHemiSphere((*args.add(0)).as_f64() as f32, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshCylinder(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshCylinder expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshCylinder((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshCone(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshCone expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshCone((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshTorus(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshTorus expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshTorus((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshKnot(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshKnot expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshKnot((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshHeightmap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshHeightmap expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshHeightmap(unpack_image(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GenMeshCubicmap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GenMeshCubicmap expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GenMeshCubicmap(unpack_image(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_mesh(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadMaterials(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadMaterials expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadMaterials(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadMaterialDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = LoadMaterialDefault();
    SolixCValue::from_ref(pack_material(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMaterialValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMaterialValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMaterialValid(unpack_material(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadMaterial(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadMaterial expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadMaterial(unpack_material(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMaterialTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMaterialTexture expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let material_addr_0 = (*args.add(0)).as_ref();
    let mut material_mut_0 = unpack_material(vm_context, material_addr_0);
    SetMaterialTexture(&mut material_mut_0, (*args.add(1)).as_i32() as c_int, unpack_texture(vm_context, (*args.add(2)).as_ref()));
    writeback_material(vm_context, material_addr_0, &material_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetModelMeshMaterial(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetModelMeshMaterial expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let model_addr_0 = (*args.add(0)).as_ref();
    let mut model_mut_0 = unpack_model(vm_context, model_addr_0);
    SetModelMeshMaterial(&mut model_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    writeback_model(vm_context, model_addr_0, &model_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadModelAnimations(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadModelAnimations expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    let call_result = LoadModelAnimations(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateModelAnimation(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateModelAnimation expected 3 arguments");
        }
        return SolixCValue::null();
    }
    UpdateModelAnimation(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_model_animation(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateModelAnimationEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateModelAnimationEx expected 6 arguments");
        }
        return SolixCValue::null();
    }
    UpdateModelAnimationEx(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_model_animation(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32, unpack_model_animation(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32, (*args.add(5)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadModelAnimations(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadModelAnimations expected 2 arguments");
        }
        return SolixCValue::null();
    }
    UnloadModelAnimations((*args.add(0)).payload as usize as *mut C_ModelAnimation, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsModelAnimationValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsModelAnimationValid expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsModelAnimationValid(unpack_model(vm_context, (*args.add(0)).as_ref()), unpack_model_animation(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionSpheres(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionSpheres expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionSpheres(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, unpack_vector3(vm_context, (*args.add(2)).as_ref()), (*args.add(3)).as_f64() as f32);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionBoxes(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionBoxes expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionBoxes(unpack_bounding_box(vm_context, (*args.add(0)).as_ref()), unpack_bounding_box(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CheckCollisionBoxSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "CheckCollisionBoxSphere expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = CheckCollisionBoxSphere(unpack_bounding_box(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRayCollisionSphere(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetRayCollisionSphere expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetRayCollisionSphere(unpack_ray(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRayCollisionBox(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetRayCollisionBox expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetRayCollisionBox(unpack_ray(vm_context, (*args.add(0)).as_ref()), unpack_bounding_box(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRayCollisionMesh(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetRayCollisionMesh expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetRayCollisionMesh(unpack_ray(vm_context, (*args.add(0)).as_ref()), unpack_mesh(vm_context, (*args.add(1)).as_ref()), unpack_matrix(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRayCollisionTriangle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetRayCollisionTriangle expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetRayCollisionTriangle(unpack_ray(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), unpack_vector3(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::from_ref(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetRayCollisionQuad(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetRayCollisionQuad expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetRayCollisionQuad(unpack_ray(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), unpack_vector3(vm_context, (*args.add(3)).as_ref()), unpack_vector3(vm_context, (*args.add(4)).as_ref()));
    SolixCValue::from_ref(pack_ray_collision(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_InitAudioDevice(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    InitAudioDevice();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_CloseAudioDevice(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    CloseAudioDevice();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsAudioDeviceReady(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = IsAudioDeviceReady();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMasterVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMasterVolume expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetMasterVolume((*args.add(0)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMasterVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = GetMasterVolume();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadWave expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadWave(str_ptr_0);
    SolixCValue::from_ref(pack_wave(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadWaveFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadWaveFromMemory expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<u8> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as u8);
            }
        }
    }
    let call_result = LoadWaveFromMemory(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_wave(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsWaveValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsWaveValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsWaveValid(unpack_wave(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadSound expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadSound(str_ptr_0);
    SolixCValue::from_ref(pack_sound(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadSoundFromWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadSoundFromWave expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadSoundFromWave(unpack_wave(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_sound(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadSoundAlias(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadSoundAlias expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadSoundAlias(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_sound(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsSoundValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsSoundValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsSoundValid(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateSound expected 3 arguments");
        }
        return SolixCValue::null();
    }
    UpdateSound(unpack_sound(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadWave expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadWave(unpack_wave(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadSound expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadSound(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadSoundAlias(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadSoundAlias expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadSoundAlias(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportWave(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportWave expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportWave(unpack_wave(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ExportWaveAsCode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ExportWaveAsCode expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = ExportWaveAsCode(unpack_wave(vm_context, (*args.add(0)).as_ref()), str_ptr_1);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PlaySound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PlaySound expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PlaySound(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_StopSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "StopSound expected 1 arguments");
        }
        return SolixCValue::null();
    }
    StopSound(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PauseSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PauseSound expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PauseSound(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ResumeSound(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ResumeSound expected 1 arguments");
        }
        return SolixCValue::null();
    }
    ResumeSound(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsSoundPlaying(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsSoundPlaying expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsSoundPlaying(unpack_sound(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetSoundVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetSoundVolume expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetSoundVolume(unpack_sound(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetSoundPitch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetSoundPitch expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetSoundPitch(unpack_sound(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetSoundPan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetSoundPan expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetSoundPan(unpack_sound(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_WaveCopy(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "WaveCopy expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = WaveCopy(unpack_wave(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_wave(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_WaveCrop(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "WaveCrop expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let wave_addr_0 = (*args.add(0)).as_ref();
    let mut wave_mut_0 = unpack_wave(vm_context, wave_addr_0);
    WaveCrop(&mut wave_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    writeback_wave(vm_context, wave_addr_0, &wave_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_WaveFormat(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "WaveFormat expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let wave_addr_0 = (*args.add(0)).as_ref();
    let mut wave_mut_0 = unpack_wave(vm_context, wave_addr_0);
    WaveFormat(&mut wave_mut_0, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    writeback_wave(vm_context, wave_addr_0, &wave_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadWaveSamples(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadWaveSamples expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadWaveSamples(unpack_wave(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadWaveSamples(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadWaveSamples expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadWaveSamples((*args.add(0)).payload as usize as *mut f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = LoadMusicStream(str_ptr_0);
    SolixCValue::from_ref(pack_music(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadMusicStreamFromMemory(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadMusicStreamFromMemory expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<u8> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as u8);
            }
        }
    }
    let call_result = LoadMusicStreamFromMemory(str_ptr_0, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_ref(pack_music(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMusicValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMusicValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMusicValid(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PlayMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PlayMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PlayMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsMusicStreamPlaying(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsMusicStreamPlaying expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsMusicStreamPlaying(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UpdateMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_StopMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "StopMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    StopMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PauseMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PauseMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PauseMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ResumeMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ResumeMusicStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    ResumeMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SeekMusicStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SeekMusicStream expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SeekMusicStream(unpack_music(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMusicVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMusicVolume expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetMusicVolume(unpack_music(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMusicPitch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMusicPitch expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetMusicPitch(unpack_music(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetMusicPan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetMusicPan expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetMusicPan(unpack_music(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMusicTimeLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMusicTimeLength expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMusicTimeLength(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_GetMusicTimePlayed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "GetMusicTimePlayed expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = GetMusicTimePlayed(unpack_music(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_LoadAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "LoadAudioStream expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = LoadAudioStream((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint, (*args.add(2)).payload as c_uint);
    SolixCValue::from_ref(pack_audio_stream(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsAudioStreamValid(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsAudioStreamValid expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsAudioStreamValid(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UnloadAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UnloadAudioStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    UnloadAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_UpdateAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "UpdateAudioStream expected 3 arguments");
        }
        return SolixCValue::null();
    }
    UpdateAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsAudioStreamProcessed(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsAudioStreamProcessed expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsAudioStreamProcessed(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PlayAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PlayAudioStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PlayAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_PauseAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "PauseAudioStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    PauseAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_ResumeAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "ResumeAudioStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    ResumeAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_IsAudioStreamPlaying(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "IsAudioStreamPlaying expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = IsAudioStreamPlaying(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_StopAudioStream(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "StopAudioStream expected 1 arguments");
        }
        return SolixCValue::null();
    }
    StopAudioStream(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAudioStreamVolume(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAudioStreamVolume expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetAudioStreamVolume(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAudioStreamPitch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAudioStreamPitch expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetAudioStreamPitch(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAudioStreamPan(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAudioStreamPan expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetAudioStreamPan(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAudioStreamBufferSizeDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAudioStreamBufferSizeDefault expected 1 arguments");
        }
        return SolixCValue::null();
    }
    SetAudioStreamBufferSizeDefault((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_SetAudioStreamCallback(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "SetAudioStreamCallback expected 2 arguments");
        }
        return SolixCValue::null();
    }
    SetAudioStreamCallback(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), if (*args.add(1)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(1)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_AttachAudioStreamProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "AttachAudioStreamProcessor expected 2 arguments");
        }
        return SolixCValue::null();
    }
    AttachAudioStreamProcessor(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), if (*args.add(1)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(1)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DetachAudioStreamProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DetachAudioStreamProcessor expected 2 arguments");
        }
        return SolixCValue::null();
    }
    DetachAudioStreamProcessor(unpack_audio_stream(vm_context, (*args.add(0)).as_ref()), if (*args.add(1)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(1)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_AttachAudioMixedProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "AttachAudioMixedProcessor expected 1 arguments");
        }
        return SolixCValue::null();
    }
    AttachAudioMixedProcessor(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_DetachAudioMixedProcessor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "DetachAudioMixedProcessor expected 1 arguments");
        }
        return SolixCValue::null();
    }
    DetachAudioMixedProcessor(if (*args.add(0)).payload == 0 { None } else { Some(std::mem::transmute((*args.add(0)).payload as usize)) });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlMatrixMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlMatrixMode expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlMatrixMode((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlPushMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlPushMatrix();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlPopMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlPopMatrix();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadIdentity(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlLoadIdentity();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlTranslatef(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlTranslatef expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlTranslatef((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlRotatef(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlRotatef expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlRotatef((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlScalef(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlScalef expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlScalef((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlMultMatrixf(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlMultMatrixf expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_0 = (*args.add(0)).as_ref();
    let mut prim_vec_0: Vec<f32> = Vec::new();
    if arr_addr_0 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_0);
            prim_vec_0.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_0, elem_idx);
                prim_vec_0.push(if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 });
            }
        }
    }
    rlMultMatrixf(if arr_addr_0 > 0 { prim_vec_0.as_mut_ptr() } else { std::ptr::null_mut() });
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlFrustum(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlFrustum expected 6 arguments");
        }
        return SolixCValue::null();
    }
    rlFrustum((*args.add(0)).as_f64(), (*args.add(1)).as_f64(), (*args.add(2)).as_f64(), (*args.add(3)).as_f64(), (*args.add(4)).as_f64(), (*args.add(5)).as_f64());
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlOrtho(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlOrtho expected 6 arguments");
        }
        return SolixCValue::null();
    }
    rlOrtho((*args.add(0)).as_f64(), (*args.add(1)).as_f64(), (*args.add(2)).as_f64(), (*args.add(3)).as_f64(), (*args.add(4)).as_f64(), (*args.add(5)).as_f64());
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlViewport(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlViewport expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlViewport((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetClipPlanes(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetClipPlanes expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlSetClipPlanes((*args.add(0)).as_f64(), (*args.add(1)).as_f64());
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetCullDistanceNear(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetCullDistanceNear();
    SolixCValue::from_f64(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetCullDistanceFar(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetCullDistanceFar();
    SolixCValue::from_f64(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlBegin(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlBegin expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlBegin((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnd(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnd();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlVertex2i(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlVertex2i expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlVertex2i((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlVertex2f(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlVertex2f expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlVertex2f((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlVertex3f(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlVertex3f expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlVertex3f((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlTexCoord2f(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlTexCoord2f expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlTexCoord2f((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlNormal3f(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlNormal3f expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlNormal3f((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlColor4ub(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlColor4ub expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlColor4ub((*args.add(0)).payload as u8, (*args.add(1)).payload as u8, (*args.add(2)).payload as u8, (*args.add(3)).payload as u8);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlColor3f(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlColor3f expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlColor3f((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlColor4f(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlColor4f expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlColor4f((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableVertexArray(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableVertexArray expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlEnableVertexArray((*args.add(0)).payload as c_uint);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableVertexArray(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableVertexArray();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableVertexBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableVertexBuffer expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableVertexBuffer((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableVertexBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableVertexBuffer();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableVertexBufferElement(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableVertexBufferElement expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableVertexBufferElement((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableVertexBufferElement(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableVertexBufferElement();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableVertexAttribute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableVertexAttribute expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableVertexAttribute((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableVertexAttribute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDisableVertexAttribute expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlDisableVertexAttribute((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableStatePointer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableStatePointer expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableStatePointer((*args.add(0)).as_i32() as c_int, (*args.add(1)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableStatePointer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDisableStatePointer expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlDisableStatePointer((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlActiveTextureSlot(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlActiveTextureSlot expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlActiveTextureSlot((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableTexture((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableTexture();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableTextureCubemap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableTextureCubemap expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableTextureCubemap((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableTextureCubemap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableTextureCubemap();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlTextureParameters(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlTextureParameters expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlTextureParameters((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlCubemapParameters(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlCubemapParameters expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlCubemapParameters((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableShader expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableShader((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableShader();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlEnableFramebuffer expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlEnableFramebuffer((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableFramebuffer();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetActiveFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetActiveFramebuffer();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlActiveDrawBuffers(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlActiveDrawBuffers expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlActiveDrawBuffers((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlBlitFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 9 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlBlitFramebuffer expected 9 arguments");
        }
        return SolixCValue::null();
    }
    rlBlitFramebuffer((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_i32() as c_int, (*args.add(6)).as_i32() as c_int, (*args.add(7)).as_i32() as c_int, (*args.add(8)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlBindFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlBindFramebuffer expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlBindFramebuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableColorBlend(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableColorBlend();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableColorBlend(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableColorBlend();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableDepthTest(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableDepthTest();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableDepthTest(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableDepthTest();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableDepthMask(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableDepthMask();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableDepthMask(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableDepthMask();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableBackfaceCulling(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableBackfaceCulling();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableBackfaceCulling(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableBackfaceCulling();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlColorMask(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlColorMask expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlColorMask((*args.add(0)).as_bool(), (*args.add(1)).as_bool(), (*args.add(2)).as_bool(), (*args.add(3)).as_bool());
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetCullFace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetCullFace expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetCullFace((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableScissorTest(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableScissorTest();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableScissorTest(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableScissorTest();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlScissor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlScissor expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlScissor((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnablePointMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnablePointMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisablePointMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisablePointMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetPointSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetPointSize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetPointSize((*args.add(0)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetPointSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetPointSize();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableWireMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableWireMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableWireMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableWireMode();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetLineWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetLineWidth expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetLineWidth((*args.add(0)).as_f64() as f32);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetLineWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetLineWidth();
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableSmoothLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableSmoothLines();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableSmoothLines(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableSmoothLines();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlEnableStereoRender(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlEnableStereoRender();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDisableStereoRender(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDisableStereoRender();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlIsStereoRenderEnabled(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlIsStereoRenderEnabled();
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlClearColor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlClearColor expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlClearColor((*args.add(0)).payload as u8, (*args.add(1)).payload as u8, (*args.add(2)).payload as u8, (*args.add(3)).payload as u8);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlClearScreenBuffers(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlClearScreenBuffers();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlCheckErrors(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlCheckErrors();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetBlendMode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetBlendMode expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetBlendMode((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetBlendFactors(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetBlendFactors expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlSetBlendFactors((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetBlendFactorsSeparate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetBlendFactorsSeparate expected 6 arguments");
        }
        return SolixCValue::null();
    }
    rlSetBlendFactorsSeparate((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlglInit(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlglInit expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlglInit((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlglClose(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlglClose();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadExtensions(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadExtensions expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlLoadExtensions((*args.add(0)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetProcAddress(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetProcAddress expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = rlGetProcAddress(str_ptr_0);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetVersion(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetVersion();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetFramebufferWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetFramebufferWidth expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetFramebufferWidth((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetFramebufferWidth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetFramebufferWidth();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetFramebufferHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetFramebufferHeight expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetFramebufferHeight((*args.add(0)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetFramebufferHeight(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetFramebufferHeight();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetTextureIdDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetTextureIdDefault();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetShaderIdDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetShaderIdDefault();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetShaderLocsDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetShaderLocsDefault();
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadRenderBatch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadRenderBatch expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadRenderBatch((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_ref(pack_rl_render_batch(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadRenderBatch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadRenderBatch expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadRenderBatch(unpack_rl_render_batch(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDrawRenderBatch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDrawRenderBatch expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let rl_render_batch_addr_0 = (*args.add(0)).as_ref();
    let mut rl_render_batch_mut_0 = unpack_rl_render_batch(vm_context, rl_render_batch_addr_0);
    rlDrawRenderBatch(&mut rl_render_batch_mut_0);
    writeback_rl_render_batch(vm_context, rl_render_batch_addr_0, &rl_render_batch_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetRenderBatchActive(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetRenderBatchActive expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let rl_render_batch_addr_0 = (*args.add(0)).as_ref();
    let mut rl_render_batch_mut_0 = unpack_rl_render_batch(vm_context, rl_render_batch_addr_0);
    rlSetRenderBatchActive(&mut rl_render_batch_mut_0);
    writeback_rl_render_batch(vm_context, rl_render_batch_addr_0, &rl_render_batch_mut_0);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDrawRenderBatchActive(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlDrawRenderBatchActive();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlCheckRenderBatchLimit(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlCheckRenderBatchLimit expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlCheckRenderBatchLimit((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetTexture((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadVertexArray(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlLoadVertexArray();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadVertexBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadVertexBuffer expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadVertexBuffer((*args.add(0)).payload as usize as *mut c_void, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_bool());
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadVertexBufferElement(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadVertexBufferElement expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadVertexBufferElement((*args.add(0)).payload as usize as *mut c_void, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_bool());
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUpdateVertexBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUpdateVertexBuffer expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlUpdateVertexBuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUpdateVertexBufferElements(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUpdateVertexBufferElements expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlUpdateVertexBufferElements((*args.add(0)).payload as c_uint, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadVertexArray(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadVertexArray expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadVertexArray((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadVertexBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadVertexBuffer expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadVertexBuffer((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetVertexAttribute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetVertexAttribute expected 6 arguments");
        }
        return SolixCValue::null();
    }
    rlSetVertexAttribute((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_bool(), (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetVertexAttributeDivisor(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetVertexAttributeDivisor expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlSetVertexAttributeDivisor((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetVertexAttributeDefault(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetVertexAttributeDefault expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlSetVertexAttributeDefault((*args.add(0)).as_i32() as c_int, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDrawVertexArray(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDrawVertexArray expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlDrawVertexArray((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDrawVertexArrayElements(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDrawVertexArrayElements expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlDrawVertexArrayElements((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDrawVertexArrayInstanced(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDrawVertexArrayInstanced expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlDrawVertexArrayInstanced((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlDrawVertexArrayElementsInstanced(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlDrawVertexArrayElementsInstanced expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlDrawVertexArrayElementsInstanced((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).payload as usize as *mut c_void, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadTexture expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadTexture((*args.add(0)).payload as usize as *mut c_void, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadTextureDepth(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadTextureDepth expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadTextureDepth((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_bool());
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadTextureCubemap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadTextureCubemap expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadTextureCubemap((*args.add(0)).payload as usize as *mut c_void, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUpdateTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 7 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUpdateTexture expected 7 arguments");
        }
        return SolixCValue::null();
    }
    rlUpdateTexture((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, (*args.add(5)).as_i32() as c_int, (*args.add(6)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetGlTextureFormats(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetGlTextureFormats expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_uint> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_uint);
            }
        }
    }
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<c_uint> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(raw_val as c_uint);
            }
        }
    }
    let arr_addr_3 = (*args.add(3)).as_ref();
    let mut prim_vec_3: Vec<c_uint> = Vec::new();
    if arr_addr_3 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_3);
            prim_vec_3.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_3, elem_idx);
                prim_vec_3.push(raw_val as c_uint);
            }
        }
    }
    rlGetGlTextureFormats((*args.add(0)).as_i32() as c_int, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() }, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() }, if arr_addr_3 > 0 { prim_vec_3.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, updated_elem as u64);
            }
        }
    }
    if arr_addr_3 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_3);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_3[elem_idx as usize];
                ctx.array_set_u64(arr_addr_3, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetPixelFormatName(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetPixelFormatName expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlGetPixelFormatName((*args.add(0)).payload as c_uint);
    if call_result.is_null() { return SolixCValue::null(); }
    let Some(ctx) = SolixVmContext::validate(vm_context) else { return SolixCValue::null(); };
    let c_str = CStr::from_ptr(call_result);
    let sanitized_str = c_str.to_string_lossy();
    let addr = ctx.alloc_string(sanitized_str.as_ref());
    if addr == 0 { return SolixCValue::null(); }
    SolixCValue::from_ref(addr)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadTexture expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadTexture((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGenTextureMipmaps(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGenTextureMipmaps expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_4 = (*args.add(4)).as_ref();
    let mut prim_vec_4: Vec<c_int> = Vec::new();
    if arr_addr_4 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_4);
            prim_vec_4.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_4, elem_idx);
                prim_vec_4.push(raw_val as c_int);
            }
        }
    }
    rlGenTextureMipmaps((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, if arr_addr_4 > 0 { prim_vec_4.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_4 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_4);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_4[elem_idx as usize];
                ctx.array_set_u64(arr_addr_4, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlReadTexturePixels(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlReadTexturePixels expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlReadTexturePixels((*args.add(0)).payload as c_uint, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlReadScreenPixels(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlReadScreenPixels expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlReadScreenPixels((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i64(call_result as usize as i64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlLoadFramebuffer();
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlFramebufferAttach(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlFramebufferAttach expected 5 arguments");
        }
        return SolixCValue::null();
    }
    rlFramebufferAttach((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlFramebufferComplete(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlFramebufferComplete expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlFramebufferComplete((*args.add(0)).payload as c_uint);
    SolixCValue::from_bool(call_result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadFramebuffer expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadFramebuffer((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlCopyFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlCopyFramebuffer expected 6 arguments");
        }
        return SolixCValue::null();
    }
    rlCopyFramebuffer((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int, (*args.add(4)).as_i32() as c_int, (*args.add(5)).payload as usize as *mut c_void);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlResizeFramebuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlResizeFramebuffer expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlResizeFramebuffer((*args.add(0)).as_i32() as c_int, (*args.add(1)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadShader expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = rlLoadShader(str_ptr_0, (*args.add(1)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadShaderProgram(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadShaderProgram expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_0 = (*args.add(0)).as_ref();
    let str_keepalive_0: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_0).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_0 = str_keepalive_0.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = rlLoadShaderProgram(str_ptr_0, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadShaderProgramEx(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadShaderProgramEx expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadShaderProgramEx((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadShaderProgramCompute(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadShaderProgramCompute expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadShaderProgramCompute((*args.add(0)).payload as c_uint);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadShader expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadShader((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadShaderProgram(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadShaderProgram expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadShaderProgram((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetLocationUniform(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetLocationUniform expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = rlGetLocationUniform((*args.add(0)).payload as c_uint, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetLocationAttrib(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetLocationAttrib expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let str_addr_1 = (*args.add(1)).as_ref();
    let str_keepalive_1: Option<CString> = if let Some(ctx) = SolixVmContext::validate(vm_context) {
        ctx.read_string(str_addr_1).and_then(|decoded_str| CString::new(decoded_str).ok())
    } else {
        None
    };
    let str_ptr_1 = str_keepalive_1.as_ref().map_or(c"".as_ptr(), |c_string| c_string.as_ptr());
    let call_result = rlGetLocationAttrib((*args.add(0)).payload as c_uint, str_ptr_1);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetUniform(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetUniform expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlSetUniform((*args.add(0)).as_i32() as c_int, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetUniformMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetUniformMatrix expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlSetUniformMatrix((*args.add(0)).as_i32() as c_int, unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetUniformMatrices(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetUniformMatrices expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut struct_vec_1: Vec<C_Matrix> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            struct_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let child_addr = ctx.array_get_u64(arr_addr_1, elem_idx) as u32;
                struct_vec_1.push(unpack_matrix(vm_context, child_addr));
            }
        }
    }
    rlSetUniformMatrices((*args.add(0)).as_i32() as c_int, if arr_addr_1 > 0 { struct_vec_1.as_ptr() } else { std::ptr::null() }, (*args.add(2)).as_i32() as c_int);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetUniformSampler(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetUniformSampler expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlSetUniformSampler((*args.add(0)).as_i32() as c_int, (*args.add(1)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetShader(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetShader expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let arr_addr_1 = (*args.add(1)).as_ref();
    let mut prim_vec_1: Vec<c_int> = Vec::new();
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            prim_vec_1.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_1, elem_idx);
                prim_vec_1.push(raw_val as c_int);
            }
        }
    }
    rlSetShader((*args.add(0)).payload as c_uint, if arr_addr_1 > 0 { prim_vec_1.as_mut_ptr() } else { std::ptr::null_mut() });
    if arr_addr_1 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_1);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_1[elem_idx as usize];
                ctx.array_set_u64(arr_addr_1, elem_idx, updated_elem as u64);
            }
        }
    }
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlComputeShaderDispatch(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlComputeShaderDispatch expected 3 arguments");
        }
        return SolixCValue::null();
    }
    rlComputeShaderDispatch((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint, (*args.add(2)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadShaderBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlLoadShaderBuffer expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlLoadShaderBuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).as_i32() as c_int);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUnloadShaderBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUnloadShaderBuffer expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlUnloadShaderBuffer((*args.add(0)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlUpdateShaderBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlUpdateShaderBuffer expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlUpdateShaderBuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).payload as c_uint, (*args.add(3)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlBindShaderBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlBindShaderBuffer expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlBindShaderBuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlReadShaderBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlReadShaderBuffer expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlReadShaderBuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as usize as *mut c_void, (*args.add(2)).payload as c_uint, (*args.add(3)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlCopyShaderBuffer(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlCopyShaderBuffer expected 5 arguments");
        }
        return SolixCValue::null();
    }
    rlCopyShaderBuffer((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint, (*args.add(2)).payload as c_uint, (*args.add(3)).payload as c_uint, (*args.add(4)).payload as c_uint);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetShaderBufferSize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetShaderBufferSize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlGetShaderBufferSize((*args.add(0)).payload as c_uint);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlBindImageTexture(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlBindImageTexture expected 4 arguments");
        }
        return SolixCValue::null();
    }
    rlBindImageTexture((*args.add(0)).payload as c_uint, (*args.add(1)).payload as c_uint, (*args.add(2)).as_i32() as c_int, (*args.add(3)).as_bool());
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetMatrixModelview(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetMatrixModelview();
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetMatrixProjection(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetMatrixProjection();
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetMatrixTransform(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = rlGetMatrixTransform();
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetMatrixProjectionStereo(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetMatrixProjectionStereo expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlGetMatrixProjectionStereo((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlGetMatrixViewOffsetStereo(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlGetMatrixViewOffsetStereo expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = rlGetMatrixViewOffsetStereo((*args.add(0)).as_i32() as c_int);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetMatrixProjection(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetMatrixProjection expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetMatrixProjection(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetMatrixModelview(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetMatrixModelview expected 1 arguments");
        }
        return SolixCValue::null();
    }
    rlSetMatrixModelview(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetMatrixProjectionStereo(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetMatrixProjectionStereo expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlSetMatrixProjectionStereo(unpack_matrix(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlSetMatrixViewOffsetStereo(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "rlSetMatrixViewOffsetStereo expected 2 arguments");
        }
        return SolixCValue::null();
    }
    rlSetMatrixViewOffsetStereo(unpack_matrix(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadDrawCube(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlLoadDrawCube();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_rlLoadDrawQuad(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    rlLoadDrawQuad();
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Clamp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Clamp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Clamp((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Lerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Lerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Lerp((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Normalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Normalize expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Normalize((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Remap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Remap expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Remap((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32, (*args.add(3)).as_f64() as f32, (*args.add(4)).as_f64() as f32);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Wrap(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Wrap expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Wrap((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_FloatEquals(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "FloatEquals expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = FloatEquals((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32);
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Zero(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Vector2Zero();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2One(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Vector2One();
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Add(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Add expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Add(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2AddValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2AddValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2AddValue(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Subtract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Subtract expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Subtract(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2SubtractValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2SubtractValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2SubtractValue(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Length(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Length expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Length(unpack_vector2(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2LengthSqr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2LengthSqr expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2LengthSqr(unpack_vector2(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2DotProduct(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2DotProduct expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2DotProduct(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2CrossProduct(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2CrossProduct expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2CrossProduct(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Distance(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Distance expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Distance(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2DistanceSqr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2DistanceSqr expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2DistanceSqr(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Angle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Angle expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Angle(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2LineAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2LineAngle expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2LineAngle(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Scale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Scale expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Scale(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Multiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Multiply expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Multiply(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Negate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Negate expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Negate(unpack_vector2(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Divide(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Divide expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Divide(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Normalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Normalize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Normalize(unpack_vector2(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Transform(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Transform expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Transform(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Lerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Lerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Lerp(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Reflect(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Reflect expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Reflect(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Min(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Min expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Min(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Max(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Max expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Max(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Rotate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Rotate expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Rotate(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2MoveTowards(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2MoveTowards expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2MoveTowards(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Invert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Invert expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Invert(unpack_vector2(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Clamp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Clamp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Clamp(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), unpack_vector2(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2ClampValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2ClampValue expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2ClampValue(unpack_vector2(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Equals(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Equals expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Equals(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector2Refract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector2Refract expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector2Refract(unpack_vector2(vm_context, (*args.add(0)).as_ref()), unpack_vector2(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector2(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Zero(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Vector3Zero();
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3One(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Vector3One();
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Add(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Add expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Add(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3AddValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3AddValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3AddValue(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Subtract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Subtract expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Subtract(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3SubtractValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3SubtractValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3SubtractValue(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Scale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Scale expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Scale(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Multiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Multiply expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Multiply(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3CrossProduct(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3CrossProduct expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3CrossProduct(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Perpendicular(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Perpendicular expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Perpendicular(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Length(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Length expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Length(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3LengthSqr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3LengthSqr expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3LengthSqr(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3DotProduct(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3DotProduct expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3DotProduct(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Distance(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Distance expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Distance(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3DistanceSqr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3DistanceSqr expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3DistanceSqr(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Angle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Angle expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Angle(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Negate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Negate expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Negate(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Divide(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Divide expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Divide(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Normalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Normalize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Normalize(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Project(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Project expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Project(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Reject(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Reject expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Reject(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3OrthoNormalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3OrthoNormalize expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let vector3_addr_0 = (*args.add(0)).as_ref();
    let mut vector3_mut_0 = unpack_vector3(vm_context, vector3_addr_0);
    let vector3_addr_1 = (*args.add(1)).as_ref();
    let mut vector3_mut_1 = unpack_vector3(vm_context, vector3_addr_1);
    Vector3OrthoNormalize(&mut vector3_mut_0, &mut vector3_mut_1);
    writeback_vector3(vm_context, vector3_addr_0, &vector3_mut_0);
    writeback_vector3(vm_context, vector3_addr_1, &vector3_mut_1);
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Transform(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Transform expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Transform(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3RotateByQuaternion(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3RotateByQuaternion expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3RotateByQuaternion(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3RotateByAxisAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3RotateByAxisAngle expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3RotateByAxisAngle(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3MoveTowards(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3MoveTowards expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3MoveTowards(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Lerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Lerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Lerp(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3CubicHermite(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3CubicHermite expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3CubicHermite(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), unpack_vector3(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Reflect(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Reflect expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Reflect(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Min(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Min expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Min(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Max(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Max expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Max(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Barycenter(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Barycenter expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Barycenter(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()), unpack_vector3(vm_context, (*args.add(3)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Unproject(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Unproject expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Unproject(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()), unpack_matrix(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3ToFloatV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3ToFloatV expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3ToFloatV(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_float3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Invert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Invert expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Invert(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Clamp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Clamp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Clamp(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3ClampValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3ClampValue expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3ClampValue(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Equals(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Equals expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Equals(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector3Refract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector3Refract expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector3Refract(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Zero(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Vector4Zero();
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4One(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = Vector4One();
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Add(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Add expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Add(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4AddValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4AddValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4AddValue(unpack_vector4(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Subtract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Subtract expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Subtract(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4SubtractValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4SubtractValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4SubtractValue(unpack_vector4(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Length(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Length expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Length(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4LengthSqr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4LengthSqr expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4LengthSqr(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4DotProduct(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4DotProduct expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4DotProduct(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Distance(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Distance expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Distance(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4DistanceSqr(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4DistanceSqr expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4DistanceSqr(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Scale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Scale expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Scale(unpack_vector4(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Multiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Multiply expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Multiply(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Negate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Negate expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Negate(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Divide(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Divide expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Divide(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Normalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Normalize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Normalize(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Min(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Min expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Min(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Max(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Max expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Max(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Lerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Lerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Lerp(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4MoveTowards(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4MoveTowards expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4MoveTowards(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Invert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Invert expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Invert(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_Vector4Equals(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "Vector4Equals expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = Vector4Equals(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixDeterminant(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixDeterminant expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixDeterminant(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixTrace(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixTrace expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixTrace(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixTranspose(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixTranspose expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixTranspose(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixInvert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixInvert expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixInvert(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixIdentity(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = MatrixIdentity();
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixAdd(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixAdd expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixAdd(unpack_matrix(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixSubtract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixSubtract expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixSubtract(unpack_matrix(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixMultiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixMultiply expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixMultiply(unpack_matrix(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixMultiplyValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixMultiplyValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixMultiplyValue(unpack_matrix(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixTranslate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixTranslate expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixTranslate((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixRotate(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixRotate expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixRotate(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixRotateX(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixRotateX expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixRotateX((*args.add(0)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixRotateY(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixRotateY expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixRotateY((*args.add(0)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixRotateZ(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixRotateZ expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixRotateZ((*args.add(0)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixRotateXYZ(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixRotateXYZ expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixRotateXYZ(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixRotateZYX(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixRotateZYX expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixRotateZYX(unpack_vector3(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixScale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixScale expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixScale((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixFrustum(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixFrustum expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixFrustum((*args.add(0)).as_f64(), (*args.add(1)).as_f64(), (*args.add(2)).as_f64(), (*args.add(3)).as_f64(), (*args.add(4)).as_f64(), (*args.add(5)).as_f64());
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixPerspective(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixPerspective expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixPerspective((*args.add(0)).as_f64(), (*args.add(1)).as_f64(), (*args.add(2)).as_f64(), (*args.add(3)).as_f64());
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixOrtho(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 6 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixOrtho expected 6 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixOrtho((*args.add(0)).as_f64(), (*args.add(1)).as_f64(), (*args.add(2)).as_f64(), (*args.add(3)).as_f64(), (*args.add(4)).as_f64(), (*args.add(5)).as_f64());
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixLookAt(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixLookAt expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixLookAt(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixToFloatV(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixToFloatV expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixToFloatV(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_float16(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionAdd(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionAdd expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionAdd(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionAddValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionAddValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionAddValue(unpack_vector4(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionSubtract(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionSubtract expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionSubtract(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionSubtractValue(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionSubtractValue expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionSubtractValue(unpack_vector4(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionIdentity(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    let call_result = QuaternionIdentity();
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionLength(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionLength expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionLength(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_f64(call_result as f64)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionNormalize(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionNormalize expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionNormalize(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionInvert(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionInvert expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionInvert(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionMultiply(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionMultiply expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionMultiply(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionScale(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionScale expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionScale(unpack_vector4(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionDivide(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionDivide expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionDivide(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionLerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionLerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionLerp(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionNlerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionNlerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionNlerp(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionSlerp(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionSlerp expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionSlerp(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionCubicHermiteSpline(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 5 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionCubicHermiteSpline expected 5 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionCubicHermiteSpline(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), unpack_vector4(vm_context, (*args.add(2)).as_ref()), unpack_vector4(vm_context, (*args.add(3)).as_ref()), (*args.add(4)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionFromVector3ToVector3(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionFromVector3ToVector3 expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionFromVector3ToVector3(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector3(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionFromMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionFromMatrix expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionFromMatrix(unpack_matrix(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionToMatrix(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionToMatrix expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionToMatrix(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionFromAxisAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionFromAxisAngle expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionFromAxisAngle(unpack_vector3(vm_context, (*args.add(0)).as_ref()), (*args.add(1)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionToAxisAngle(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionToAxisAngle expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let vector3_addr_1 = (*args.add(1)).as_ref();
    let mut vector3_mut_1 = unpack_vector3(vm_context, vector3_addr_1);
    let arr_addr_2 = (*args.add(2)).as_ref();
    let mut prim_vec_2: Vec<f32> = Vec::new();
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            prim_vec_2.reserve_exact(total_count as usize);
            for elem_idx in 0..total_count {
                let raw_val = ctx.array_get_u64(arr_addr_2, elem_idx);
                prim_vec_2.push(if (raw_val >> 32) == 0 { f32::from_bits(raw_val as u32) } else { f64::from_bits(raw_val) as f32 });
            }
        }
    }
    QuaternionToAxisAngle(unpack_vector4(vm_context, (*args.add(0)).as_ref()), &mut vector3_mut_1, if arr_addr_2 > 0 { prim_vec_2.as_mut_ptr() } else { std::ptr::null_mut() });
    writeback_vector3(vm_context, vector3_addr_1, &vector3_mut_1);
    if arr_addr_2 > 0 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            let total_count = ctx.array_len(arr_addr_2);
            for elem_idx in 0..total_count {
                let updated_elem = prim_vec_2[elem_idx as usize];
                ctx.array_set_u64(arr_addr_2, elem_idx, (updated_elem as f64).to_bits());
            }
        }
    }
    SolixCValue::null()
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionFromEuler(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionFromEuler expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionFromEuler((*args.add(0)).as_f64() as f32, (*args.add(1)).as_f64() as f32, (*args.add(2)).as_f64() as f32);
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionToEuler(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 1 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionToEuler expected 1 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionToEuler(unpack_vector4(vm_context, (*args.add(0)).as_ref()));
    SolixCValue::from_ref(pack_vector3(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionTransform(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionTransform expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionTransform(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_matrix(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_ref(pack_vector4(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_QuaternionEquals(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 2 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "QuaternionEquals expected 2 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = QuaternionEquals(unpack_vector4(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()));
    SolixCValue::from_i32(call_result as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixCompose(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 3 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixCompose expected 3 arguments");
        }
        return SolixCValue::null();
    }
    let call_result = MatrixCompose(unpack_vector3(vm_context, (*args.add(0)).as_ref()), unpack_vector4(vm_context, (*args.add(1)).as_ref()), unpack_vector3(vm_context, (*args.add(2)).as_ref()));
    SolixCValue::from_ref(pack_matrix(vm_context, call_result))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Raylib_MatrixDecompose(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if args.is_null() || (arg_count as usize) < 4 {
        if let Some(ctx) = SolixVmContext::validate(vm_context) {
            ctx.set_error(out_error, "MatrixDecompose expected 4 arguments");
        }
        return SolixCValue::null();
    }
    let vector3_addr_1 = (*args.add(1)).as_ref();
    let mut vector3_mut_1 = unpack_vector3(vm_context, vector3_addr_1);
    let vector4_addr_2 = (*args.add(2)).as_ref();
    let mut vector4_mut_2 = unpack_vector4(vm_context, vector4_addr_2);
    let vector3_addr_3 = (*args.add(3)).as_ref();
    let mut vector3_mut_3 = unpack_vector3(vm_context, vector3_addr_3);
    MatrixDecompose(unpack_matrix(vm_context, (*args.add(0)).as_ref()), &mut vector3_mut_1, &mut vector4_mut_2, &mut vector3_mut_3);
    writeback_vector3(vm_context, vector3_addr_1, &vector3_mut_1);
    writeback_vector4(vm_context, vector4_addr_2, &vector4_mut_2);
    writeback_vector3(vm_context, vector3_addr_3, &vector3_mut_3);
    SolixCValue::null()
}

// ===========================================================================
// Native Symbol Dynamic Registration Hook
// ===========================================================================
solix_export! {
    "raylib::InitWindow" => Raylib_InitWindow,
    "InitWindow" => Raylib_InitWindow,
    "raylib::CloseWindow" => Raylib_CloseWindow,
    "CloseWindow" => Raylib_CloseWindow,
    "raylib::WindowShouldClose" => Raylib_WindowShouldClose,
    "WindowShouldClose" => Raylib_WindowShouldClose,
    "raylib::IsWindowReady" => Raylib_IsWindowReady,
    "IsWindowReady" => Raylib_IsWindowReady,
    "raylib::IsWindowFullscreen" => Raylib_IsWindowFullscreen,
    "IsWindowFullscreen" => Raylib_IsWindowFullscreen,
    "raylib::IsWindowHidden" => Raylib_IsWindowHidden,
    "IsWindowHidden" => Raylib_IsWindowHidden,
    "raylib::IsWindowMinimized" => Raylib_IsWindowMinimized,
    "IsWindowMinimized" => Raylib_IsWindowMinimized,
    "raylib::IsWindowMaximized" => Raylib_IsWindowMaximized,
    "IsWindowMaximized" => Raylib_IsWindowMaximized,
    "raylib::IsWindowFocused" => Raylib_IsWindowFocused,
    "IsWindowFocused" => Raylib_IsWindowFocused,
    "raylib::IsWindowResized" => Raylib_IsWindowResized,
    "IsWindowResized" => Raylib_IsWindowResized,
    "raylib::IsWindowState" => Raylib_IsWindowState,
    "IsWindowState" => Raylib_IsWindowState,
    "raylib::SetWindowState" => Raylib_SetWindowState,
    "SetWindowState" => Raylib_SetWindowState,
    "raylib::ClearWindowState" => Raylib_ClearWindowState,
    "ClearWindowState" => Raylib_ClearWindowState,
    "raylib::ToggleFullscreen" => Raylib_ToggleFullscreen,
    "ToggleFullscreen" => Raylib_ToggleFullscreen,
    "raylib::ToggleBorderlessWindowed" => Raylib_ToggleBorderlessWindowed,
    "ToggleBorderlessWindowed" => Raylib_ToggleBorderlessWindowed,
    "raylib::MaximizeWindow" => Raylib_MaximizeWindow,
    "MaximizeWindow" => Raylib_MaximizeWindow,
    "raylib::MinimizeWindow" => Raylib_MinimizeWindow,
    "MinimizeWindow" => Raylib_MinimizeWindow,
    "raylib::RestoreWindow" => Raylib_RestoreWindow,
    "RestoreWindow" => Raylib_RestoreWindow,
    "raylib::SetWindowIcon" => Raylib_SetWindowIcon,
    "SetWindowIcon" => Raylib_SetWindowIcon,
    "raylib::SetWindowIcons" => Raylib_SetWindowIcons,
    "SetWindowIcons" => Raylib_SetWindowIcons,
    "raylib::SetWindowTitle" => Raylib_SetWindowTitle,
    "SetWindowTitle" => Raylib_SetWindowTitle,
    "raylib::SetWindowPosition" => Raylib_SetWindowPosition,
    "SetWindowPosition" => Raylib_SetWindowPosition,
    "raylib::SetWindowMonitor" => Raylib_SetWindowMonitor,
    "SetWindowMonitor" => Raylib_SetWindowMonitor,
    "raylib::SetWindowMinSize" => Raylib_SetWindowMinSize,
    "SetWindowMinSize" => Raylib_SetWindowMinSize,
    "raylib::SetWindowMaxSize" => Raylib_SetWindowMaxSize,
    "SetWindowMaxSize" => Raylib_SetWindowMaxSize,
    "raylib::SetWindowSize" => Raylib_SetWindowSize,
    "SetWindowSize" => Raylib_SetWindowSize,
    "raylib::SetWindowOpacity" => Raylib_SetWindowOpacity,
    "SetWindowOpacity" => Raylib_SetWindowOpacity,
    "raylib::SetWindowFocused" => Raylib_SetWindowFocused,
    "SetWindowFocused" => Raylib_SetWindowFocused,
    "raylib::GetWindowHandle" => Raylib_GetWindowHandle,
    "GetWindowHandle" => Raylib_GetWindowHandle,
    "raylib::GetScreenWidth" => Raylib_GetScreenWidth,
    "GetScreenWidth" => Raylib_GetScreenWidth,
    "raylib::GetScreenHeight" => Raylib_GetScreenHeight,
    "GetScreenHeight" => Raylib_GetScreenHeight,
    "raylib::GetRenderWidth" => Raylib_GetRenderWidth,
    "GetRenderWidth" => Raylib_GetRenderWidth,
    "raylib::GetRenderHeight" => Raylib_GetRenderHeight,
    "GetRenderHeight" => Raylib_GetRenderHeight,
    "raylib::GetMonitorCount" => Raylib_GetMonitorCount,
    "GetMonitorCount" => Raylib_GetMonitorCount,
    "raylib::GetCurrentMonitor" => Raylib_GetCurrentMonitor,
    "GetCurrentMonitor" => Raylib_GetCurrentMonitor,
    "raylib::GetMonitorPosition" => Raylib_GetMonitorPosition,
    "GetMonitorPosition" => Raylib_GetMonitorPosition,
    "raylib::GetMonitorWidth" => Raylib_GetMonitorWidth,
    "GetMonitorWidth" => Raylib_GetMonitorWidth,
    "raylib::GetMonitorHeight" => Raylib_GetMonitorHeight,
    "GetMonitorHeight" => Raylib_GetMonitorHeight,
    "raylib::GetMonitorPhysicalWidth" => Raylib_GetMonitorPhysicalWidth,
    "GetMonitorPhysicalWidth" => Raylib_GetMonitorPhysicalWidth,
    "raylib::GetMonitorPhysicalHeight" => Raylib_GetMonitorPhysicalHeight,
    "GetMonitorPhysicalHeight" => Raylib_GetMonitorPhysicalHeight,
    "raylib::GetMonitorRefreshRate" => Raylib_GetMonitorRefreshRate,
    "GetMonitorRefreshRate" => Raylib_GetMonitorRefreshRate,
    "raylib::GetWindowPosition" => Raylib_GetWindowPosition,
    "GetWindowPosition" => Raylib_GetWindowPosition,
    "raylib::GetWindowScaleDPI" => Raylib_GetWindowScaleDPI,
    "GetWindowScaleDPI" => Raylib_GetWindowScaleDPI,
    "raylib::GetMonitorName" => Raylib_GetMonitorName,
    "GetMonitorName" => Raylib_GetMonitorName,
    "raylib::SetClipboardText" => Raylib_SetClipboardText,
    "SetClipboardText" => Raylib_SetClipboardText,
    "raylib::GetClipboardText" => Raylib_GetClipboardText,
    "GetClipboardText" => Raylib_GetClipboardText,
    "raylib::GetClipboardImage" => Raylib_GetClipboardImage,
    "GetClipboardImage" => Raylib_GetClipboardImage,
    "raylib::EnableEventWaiting" => Raylib_EnableEventWaiting,
    "EnableEventWaiting" => Raylib_EnableEventWaiting,
    "raylib::DisableEventWaiting" => Raylib_DisableEventWaiting,
    "DisableEventWaiting" => Raylib_DisableEventWaiting,
    "raylib::ShowCursor" => Raylib_ShowCursor,
    "ShowCursor" => Raylib_ShowCursor,
    "raylib::HideCursor" => Raylib_HideCursor,
    "HideCursor" => Raylib_HideCursor,
    "raylib::IsCursorHidden" => Raylib_IsCursorHidden,
    "IsCursorHidden" => Raylib_IsCursorHidden,
    "raylib::EnableCursor" => Raylib_EnableCursor,
    "EnableCursor" => Raylib_EnableCursor,
    "raylib::DisableCursor" => Raylib_DisableCursor,
    "DisableCursor" => Raylib_DisableCursor,
    "raylib::IsCursorOnScreen" => Raylib_IsCursorOnScreen,
    "IsCursorOnScreen" => Raylib_IsCursorOnScreen,
    "raylib::ClearBackground" => Raylib_ClearBackground,
    "ClearBackground" => Raylib_ClearBackground,
    "raylib::BeginDrawing" => Raylib_BeginDrawing,
    "BeginDrawing" => Raylib_BeginDrawing,
    "raylib::EndDrawing" => Raylib_EndDrawing,
    "EndDrawing" => Raylib_EndDrawing,
    "raylib::BeginMode2D" => Raylib_BeginMode2D,
    "BeginMode2D" => Raylib_BeginMode2D,
    "raylib::EndMode2D" => Raylib_EndMode2D,
    "EndMode2D" => Raylib_EndMode2D,
    "raylib::BeginMode3D" => Raylib_BeginMode3D,
    "BeginMode3D" => Raylib_BeginMode3D,
    "raylib::EndMode3D" => Raylib_EndMode3D,
    "EndMode3D" => Raylib_EndMode3D,
    "raylib::BeginTextureMode" => Raylib_BeginTextureMode,
    "BeginTextureMode" => Raylib_BeginTextureMode,
    "raylib::EndTextureMode" => Raylib_EndTextureMode,
    "EndTextureMode" => Raylib_EndTextureMode,
    "raylib::BeginShaderMode" => Raylib_BeginShaderMode,
    "BeginShaderMode" => Raylib_BeginShaderMode,
    "raylib::EndShaderMode" => Raylib_EndShaderMode,
    "EndShaderMode" => Raylib_EndShaderMode,
    "raylib::BeginBlendMode" => Raylib_BeginBlendMode,
    "BeginBlendMode" => Raylib_BeginBlendMode,
    "raylib::EndBlendMode" => Raylib_EndBlendMode,
    "EndBlendMode" => Raylib_EndBlendMode,
    "raylib::BeginScissorMode" => Raylib_BeginScissorMode,
    "BeginScissorMode" => Raylib_BeginScissorMode,
    "raylib::EndScissorMode" => Raylib_EndScissorMode,
    "EndScissorMode" => Raylib_EndScissorMode,
    "raylib::BeginVrStereoMode" => Raylib_BeginVrStereoMode,
    "BeginVrStereoMode" => Raylib_BeginVrStereoMode,
    "raylib::EndVrStereoMode" => Raylib_EndVrStereoMode,
    "EndVrStereoMode" => Raylib_EndVrStereoMode,
    "raylib::LoadVrStereoConfig" => Raylib_LoadVrStereoConfig,
    "LoadVrStereoConfig" => Raylib_LoadVrStereoConfig,
    "raylib::UnloadVrStereoConfig" => Raylib_UnloadVrStereoConfig,
    "UnloadVrStereoConfig" => Raylib_UnloadVrStereoConfig,
    "raylib::LoadShader" => Raylib_LoadShader,
    "LoadShader" => Raylib_LoadShader,
    "raylib::LoadShaderFromMemory" => Raylib_LoadShaderFromMemory,
    "LoadShaderFromMemory" => Raylib_LoadShaderFromMemory,
    "raylib::IsShaderValid" => Raylib_IsShaderValid,
    "IsShaderValid" => Raylib_IsShaderValid,
    "raylib::GetShaderLocation" => Raylib_GetShaderLocation,
    "GetShaderLocation" => Raylib_GetShaderLocation,
    "raylib::GetShaderLocationAttrib" => Raylib_GetShaderLocationAttrib,
    "GetShaderLocationAttrib" => Raylib_GetShaderLocationAttrib,
    "raylib::SetShaderValue" => Raylib_SetShaderValue,
    "SetShaderValue" => Raylib_SetShaderValue,
    "raylib::SetShaderValueV" => Raylib_SetShaderValueV,
    "SetShaderValueV" => Raylib_SetShaderValueV,
    "raylib::SetShaderValueMatrix" => Raylib_SetShaderValueMatrix,
    "SetShaderValueMatrix" => Raylib_SetShaderValueMatrix,
    "raylib::SetShaderValueTexture" => Raylib_SetShaderValueTexture,
    "SetShaderValueTexture" => Raylib_SetShaderValueTexture,
    "raylib::UnloadShader" => Raylib_UnloadShader,
    "UnloadShader" => Raylib_UnloadShader,
    "raylib::GetScreenToWorldRay" => Raylib_GetScreenToWorldRay,
    "GetScreenToWorldRay" => Raylib_GetScreenToWorldRay,
    "raylib::GetScreenToWorldRayEx" => Raylib_GetScreenToWorldRayEx,
    "GetScreenToWorldRayEx" => Raylib_GetScreenToWorldRayEx,
    "raylib::GetWorldToScreen" => Raylib_GetWorldToScreen,
    "GetWorldToScreen" => Raylib_GetWorldToScreen,
    "raylib::GetWorldToScreenEx" => Raylib_GetWorldToScreenEx,
    "GetWorldToScreenEx" => Raylib_GetWorldToScreenEx,
    "raylib::GetWorldToScreen2D" => Raylib_GetWorldToScreen2D,
    "GetWorldToScreen2D" => Raylib_GetWorldToScreen2D,
    "raylib::GetScreenToWorld2D" => Raylib_GetScreenToWorld2D,
    "GetScreenToWorld2D" => Raylib_GetScreenToWorld2D,
    "raylib::GetCameraMatrix" => Raylib_GetCameraMatrix,
    "GetCameraMatrix" => Raylib_GetCameraMatrix,
    "raylib::GetCameraMatrix2D" => Raylib_GetCameraMatrix2D,
    "GetCameraMatrix2D" => Raylib_GetCameraMatrix2D,
    "raylib::SetTargetFPS" => Raylib_SetTargetFPS,
    "SetTargetFPS" => Raylib_SetTargetFPS,
    "raylib::GetFrameTime" => Raylib_GetFrameTime,
    "GetFrameTime" => Raylib_GetFrameTime,
    "raylib::GetTime" => Raylib_GetTime,
    "GetTime" => Raylib_GetTime,
    "raylib::GetFPS" => Raylib_GetFPS,
    "GetFPS" => Raylib_GetFPS,
    "raylib::SwapScreenBuffer" => Raylib_SwapScreenBuffer,
    "SwapScreenBuffer" => Raylib_SwapScreenBuffer,
    "raylib::PollInputEvents" => Raylib_PollInputEvents,
    "PollInputEvents" => Raylib_PollInputEvents,
    "raylib::WaitTime" => Raylib_WaitTime,
    "WaitTime" => Raylib_WaitTime,
    "raylib::SetRandomSeed" => Raylib_SetRandomSeed,
    "SetRandomSeed" => Raylib_SetRandomSeed,
    "raylib::GetRandomValue" => Raylib_GetRandomValue,
    "GetRandomValue" => Raylib_GetRandomValue,
    "raylib::LoadRandomSequence" => Raylib_LoadRandomSequence,
    "LoadRandomSequence" => Raylib_LoadRandomSequence,
    "raylib::UnloadRandomSequence" => Raylib_UnloadRandomSequence,
    "UnloadRandomSequence" => Raylib_UnloadRandomSequence,
    "raylib::TakeScreenshot" => Raylib_TakeScreenshot,
    "TakeScreenshot" => Raylib_TakeScreenshot,
    "raylib::SetConfigFlags" => Raylib_SetConfigFlags,
    "SetConfigFlags" => Raylib_SetConfigFlags,
    "raylib::OpenURL" => Raylib_OpenURL,
    "OpenURL" => Raylib_OpenURL,
    "raylib::SetTraceLogLevel" => Raylib_SetTraceLogLevel,
    "SetTraceLogLevel" => Raylib_SetTraceLogLevel,
    "raylib::TraceLog" => Raylib_TraceLog,
    "TraceLog" => Raylib_TraceLog,
    "raylib::SetTraceLogCallback" => Raylib_SetTraceLogCallback,
    "SetTraceLogCallback" => Raylib_SetTraceLogCallback,
    "raylib::MemAlloc" => Raylib_MemAlloc,
    "MemAlloc" => Raylib_MemAlloc,
    "raylib::MemRealloc" => Raylib_MemRealloc,
    "MemRealloc" => Raylib_MemRealloc,
    "raylib::MemFree" => Raylib_MemFree,
    "MemFree" => Raylib_MemFree,
    "raylib::LoadFileData" => Raylib_LoadFileData,
    "LoadFileData" => Raylib_LoadFileData,
    "raylib::UnloadFileData" => Raylib_UnloadFileData,
    "UnloadFileData" => Raylib_UnloadFileData,
    "raylib::SaveFileData" => Raylib_SaveFileData,
    "SaveFileData" => Raylib_SaveFileData,
    "raylib::ExportDataAsCode" => Raylib_ExportDataAsCode,
    "ExportDataAsCode" => Raylib_ExportDataAsCode,
    "raylib::LoadFileText" => Raylib_LoadFileText,
    "LoadFileText" => Raylib_LoadFileText,
    "raylib::UnloadFileText" => Raylib_UnloadFileText,
    "UnloadFileText" => Raylib_UnloadFileText,
    "raylib::SaveFileText" => Raylib_SaveFileText,
    "SaveFileText" => Raylib_SaveFileText,
    "raylib::SetLoadFileDataCallback" => Raylib_SetLoadFileDataCallback,
    "SetLoadFileDataCallback" => Raylib_SetLoadFileDataCallback,
    "raylib::SetSaveFileDataCallback" => Raylib_SetSaveFileDataCallback,
    "SetSaveFileDataCallback" => Raylib_SetSaveFileDataCallback,
    "raylib::SetLoadFileTextCallback" => Raylib_SetLoadFileTextCallback,
    "SetLoadFileTextCallback" => Raylib_SetLoadFileTextCallback,
    "raylib::SetSaveFileTextCallback" => Raylib_SetSaveFileTextCallback,
    "SetSaveFileTextCallback" => Raylib_SetSaveFileTextCallback,
    "raylib::FileRename" => Raylib_FileRename,
    "FileRename" => Raylib_FileRename,
    "raylib::FileRemove" => Raylib_FileRemove,
    "FileRemove" => Raylib_FileRemove,
    "raylib::FileCopy" => Raylib_FileCopy,
    "FileCopy" => Raylib_FileCopy,
    "raylib::FileMove" => Raylib_FileMove,
    "FileMove" => Raylib_FileMove,
    "raylib::FileTextReplace" => Raylib_FileTextReplace,
    "FileTextReplace" => Raylib_FileTextReplace,
    "raylib::FileTextFindIndex" => Raylib_FileTextFindIndex,
    "FileTextFindIndex" => Raylib_FileTextFindIndex,
    "raylib::FileExists" => Raylib_FileExists,
    "FileExists" => Raylib_FileExists,
    "raylib::DirectoryExists" => Raylib_DirectoryExists,
    "DirectoryExists" => Raylib_DirectoryExists,
    "raylib::IsFileExtension" => Raylib_IsFileExtension,
    "IsFileExtension" => Raylib_IsFileExtension,
    "raylib::IsFileHidden" => Raylib_IsFileHidden,
    "IsFileHidden" => Raylib_IsFileHidden,
    "raylib::GetFileLength" => Raylib_GetFileLength,
    "GetFileLength" => Raylib_GetFileLength,
    "raylib::GetFileModTime" => Raylib_GetFileModTime,
    "GetFileModTime" => Raylib_GetFileModTime,
    "raylib::GetFileExtension" => Raylib_GetFileExtension,
    "GetFileExtension" => Raylib_GetFileExtension,
    "raylib::GetFileName" => Raylib_GetFileName,
    "GetFileName" => Raylib_GetFileName,
    "raylib::GetFileNameWithoutExt" => Raylib_GetFileNameWithoutExt,
    "GetFileNameWithoutExt" => Raylib_GetFileNameWithoutExt,
    "raylib::GetDirectoryPath" => Raylib_GetDirectoryPath,
    "GetDirectoryPath" => Raylib_GetDirectoryPath,
    "raylib::GetPrevDirectoryPath" => Raylib_GetPrevDirectoryPath,
    "GetPrevDirectoryPath" => Raylib_GetPrevDirectoryPath,
    "raylib::GetWorkingDirectory" => Raylib_GetWorkingDirectory,
    "GetWorkingDirectory" => Raylib_GetWorkingDirectory,
    "raylib::GetApplicationDirectory" => Raylib_GetApplicationDirectory,
    "GetApplicationDirectory" => Raylib_GetApplicationDirectory,
    "raylib::MakeDirectory" => Raylib_MakeDirectory,
    "MakeDirectory" => Raylib_MakeDirectory,
    "raylib::ChangeDirectory" => Raylib_ChangeDirectory,
    "ChangeDirectory" => Raylib_ChangeDirectory,
    "raylib::IsPathFile" => Raylib_IsPathFile,
    "IsPathFile" => Raylib_IsPathFile,
    "raylib::IsPathDirectory" => Raylib_IsPathDirectory,
    "IsPathDirectory" => Raylib_IsPathDirectory,
    "raylib::IsPathAbsolute" => Raylib_IsPathAbsolute,
    "IsPathAbsolute" => Raylib_IsPathAbsolute,
    "raylib::IsFileNameValid" => Raylib_IsFileNameValid,
    "IsFileNameValid" => Raylib_IsFileNameValid,
    "raylib::LoadDirectoryFiles" => Raylib_LoadDirectoryFiles,
    "LoadDirectoryFiles" => Raylib_LoadDirectoryFiles,
    "raylib::LoadDirectoryFilesEx" => Raylib_LoadDirectoryFilesEx,
    "LoadDirectoryFilesEx" => Raylib_LoadDirectoryFilesEx,
    "raylib::UnloadDirectoryFiles" => Raylib_UnloadDirectoryFiles,
    "UnloadDirectoryFiles" => Raylib_UnloadDirectoryFiles,
    "raylib::IsFileDropped" => Raylib_IsFileDropped,
    "IsFileDropped" => Raylib_IsFileDropped,
    "raylib::LoadDroppedFiles" => Raylib_LoadDroppedFiles,
    "LoadDroppedFiles" => Raylib_LoadDroppedFiles,
    "raylib::UnloadDroppedFiles" => Raylib_UnloadDroppedFiles,
    "UnloadDroppedFiles" => Raylib_UnloadDroppedFiles,
    "raylib::GetDirectoryFileCount" => Raylib_GetDirectoryFileCount,
    "GetDirectoryFileCount" => Raylib_GetDirectoryFileCount,
    "raylib::GetDirectoryFileCountEx" => Raylib_GetDirectoryFileCountEx,
    "GetDirectoryFileCountEx" => Raylib_GetDirectoryFileCountEx,
    "raylib::CompressData" => Raylib_CompressData,
    "CompressData" => Raylib_CompressData,
    "raylib::DecompressData" => Raylib_DecompressData,
    "DecompressData" => Raylib_DecompressData,
    "raylib::EncodeDataBase64" => Raylib_EncodeDataBase64,
    "EncodeDataBase64" => Raylib_EncodeDataBase64,
    "raylib::DecodeDataBase64" => Raylib_DecodeDataBase64,
    "DecodeDataBase64" => Raylib_DecodeDataBase64,
    "raylib::ComputeCRC32" => Raylib_ComputeCRC32,
    "ComputeCRC32" => Raylib_ComputeCRC32,
    "raylib::ComputeMD5" => Raylib_ComputeMD5,
    "ComputeMD5" => Raylib_ComputeMD5,
    "raylib::ComputeSHA1" => Raylib_ComputeSHA1,
    "ComputeSHA1" => Raylib_ComputeSHA1,
    "raylib::ComputeSHA256" => Raylib_ComputeSHA256,
    "ComputeSHA256" => Raylib_ComputeSHA256,
    "raylib::LoadAutomationEventList" => Raylib_LoadAutomationEventList,
    "LoadAutomationEventList" => Raylib_LoadAutomationEventList,
    "raylib::UnloadAutomationEventList" => Raylib_UnloadAutomationEventList,
    "UnloadAutomationEventList" => Raylib_UnloadAutomationEventList,
    "raylib::ExportAutomationEventList" => Raylib_ExportAutomationEventList,
    "ExportAutomationEventList" => Raylib_ExportAutomationEventList,
    "raylib::SetAutomationEventList" => Raylib_SetAutomationEventList,
    "SetAutomationEventList" => Raylib_SetAutomationEventList,
    "raylib::SetAutomationEventBaseFrame" => Raylib_SetAutomationEventBaseFrame,
    "SetAutomationEventBaseFrame" => Raylib_SetAutomationEventBaseFrame,
    "raylib::StartAutomationEventRecording" => Raylib_StartAutomationEventRecording,
    "StartAutomationEventRecording" => Raylib_StartAutomationEventRecording,
    "raylib::StopAutomationEventRecording" => Raylib_StopAutomationEventRecording,
    "StopAutomationEventRecording" => Raylib_StopAutomationEventRecording,
    "raylib::PlayAutomationEvent" => Raylib_PlayAutomationEvent,
    "PlayAutomationEvent" => Raylib_PlayAutomationEvent,
    "raylib::IsKeyPressed" => Raylib_IsKeyPressed,
    "IsKeyPressed" => Raylib_IsKeyPressed,
    "raylib::IsKeyPressedRepeat" => Raylib_IsKeyPressedRepeat,
    "IsKeyPressedRepeat" => Raylib_IsKeyPressedRepeat,
    "raylib::IsKeyDown" => Raylib_IsKeyDown,
    "IsKeyDown" => Raylib_IsKeyDown,
    "raylib::IsKeyReleased" => Raylib_IsKeyReleased,
    "IsKeyReleased" => Raylib_IsKeyReleased,
    "raylib::IsKeyUp" => Raylib_IsKeyUp,
    "IsKeyUp" => Raylib_IsKeyUp,
    "raylib::GetKeyPressed" => Raylib_GetKeyPressed,
    "GetKeyPressed" => Raylib_GetKeyPressed,
    "raylib::GetCharPressed" => Raylib_GetCharPressed,
    "GetCharPressed" => Raylib_GetCharPressed,
    "raylib::GetKeyName" => Raylib_GetKeyName,
    "GetKeyName" => Raylib_GetKeyName,
    "raylib::SetExitKey" => Raylib_SetExitKey,
    "SetExitKey" => Raylib_SetExitKey,
    "raylib::IsGamepadAvailable" => Raylib_IsGamepadAvailable,
    "IsGamepadAvailable" => Raylib_IsGamepadAvailable,
    "raylib::GetGamepadName" => Raylib_GetGamepadName,
    "GetGamepadName" => Raylib_GetGamepadName,
    "raylib::IsGamepadButtonPressed" => Raylib_IsGamepadButtonPressed,
    "IsGamepadButtonPressed" => Raylib_IsGamepadButtonPressed,
    "raylib::IsGamepadButtonDown" => Raylib_IsGamepadButtonDown,
    "IsGamepadButtonDown" => Raylib_IsGamepadButtonDown,
    "raylib::IsGamepadButtonReleased" => Raylib_IsGamepadButtonReleased,
    "IsGamepadButtonReleased" => Raylib_IsGamepadButtonReleased,
    "raylib::IsGamepadButtonUp" => Raylib_IsGamepadButtonUp,
    "IsGamepadButtonUp" => Raylib_IsGamepadButtonUp,
    "raylib::GetGamepadButtonPressed" => Raylib_GetGamepadButtonPressed,
    "GetGamepadButtonPressed" => Raylib_GetGamepadButtonPressed,
    "raylib::GetGamepadAxisCount" => Raylib_GetGamepadAxisCount,
    "GetGamepadAxisCount" => Raylib_GetGamepadAxisCount,
    "raylib::GetGamepadAxisMovement" => Raylib_GetGamepadAxisMovement,
    "GetGamepadAxisMovement" => Raylib_GetGamepadAxisMovement,
    "raylib::SetGamepadMappings" => Raylib_SetGamepadMappings,
    "SetGamepadMappings" => Raylib_SetGamepadMappings,
    "raylib::SetGamepadVibration" => Raylib_SetGamepadVibration,
    "SetGamepadVibration" => Raylib_SetGamepadVibration,
    "raylib::IsMouseButtonPressed" => Raylib_IsMouseButtonPressed,
    "IsMouseButtonPressed" => Raylib_IsMouseButtonPressed,
    "raylib::IsMouseButtonDown" => Raylib_IsMouseButtonDown,
    "IsMouseButtonDown" => Raylib_IsMouseButtonDown,
    "raylib::IsMouseButtonReleased" => Raylib_IsMouseButtonReleased,
    "IsMouseButtonReleased" => Raylib_IsMouseButtonReleased,
    "raylib::IsMouseButtonUp" => Raylib_IsMouseButtonUp,
    "IsMouseButtonUp" => Raylib_IsMouseButtonUp,
    "raylib::GetMouseX" => Raylib_GetMouseX,
    "GetMouseX" => Raylib_GetMouseX,
    "raylib::GetMouseY" => Raylib_GetMouseY,
    "GetMouseY" => Raylib_GetMouseY,
    "raylib::GetMousePosition" => Raylib_GetMousePosition,
    "GetMousePosition" => Raylib_GetMousePosition,
    "raylib::GetMouseDelta" => Raylib_GetMouseDelta,
    "GetMouseDelta" => Raylib_GetMouseDelta,
    "raylib::SetMousePosition" => Raylib_SetMousePosition,
    "SetMousePosition" => Raylib_SetMousePosition,
    "raylib::SetMouseOffset" => Raylib_SetMouseOffset,
    "SetMouseOffset" => Raylib_SetMouseOffset,
    "raylib::SetMouseScale" => Raylib_SetMouseScale,
    "SetMouseScale" => Raylib_SetMouseScale,
    "raylib::GetMouseWheelMove" => Raylib_GetMouseWheelMove,
    "GetMouseWheelMove" => Raylib_GetMouseWheelMove,
    "raylib::GetMouseWheelMoveV" => Raylib_GetMouseWheelMoveV,
    "GetMouseWheelMoveV" => Raylib_GetMouseWheelMoveV,
    "raylib::SetMouseCursor" => Raylib_SetMouseCursor,
    "SetMouseCursor" => Raylib_SetMouseCursor,
    "raylib::GetTouchX" => Raylib_GetTouchX,
    "GetTouchX" => Raylib_GetTouchX,
    "raylib::GetTouchY" => Raylib_GetTouchY,
    "GetTouchY" => Raylib_GetTouchY,
    "raylib::GetTouchPosition" => Raylib_GetTouchPosition,
    "GetTouchPosition" => Raylib_GetTouchPosition,
    "raylib::GetTouchPointId" => Raylib_GetTouchPointId,
    "GetTouchPointId" => Raylib_GetTouchPointId,
    "raylib::GetTouchPointCount" => Raylib_GetTouchPointCount,
    "GetTouchPointCount" => Raylib_GetTouchPointCount,
    "raylib::SetGesturesEnabled" => Raylib_SetGesturesEnabled,
    "SetGesturesEnabled" => Raylib_SetGesturesEnabled,
    "raylib::IsGestureDetected" => Raylib_IsGestureDetected,
    "IsGestureDetected" => Raylib_IsGestureDetected,
    "raylib::GetGestureDetected" => Raylib_GetGestureDetected,
    "GetGestureDetected" => Raylib_GetGestureDetected,
    "raylib::GetGestureHoldDuration" => Raylib_GetGestureHoldDuration,
    "GetGestureHoldDuration" => Raylib_GetGestureHoldDuration,
    "raylib::GetGestureDragVector" => Raylib_GetGestureDragVector,
    "GetGestureDragVector" => Raylib_GetGestureDragVector,
    "raylib::GetGestureDragAngle" => Raylib_GetGestureDragAngle,
    "GetGestureDragAngle" => Raylib_GetGestureDragAngle,
    "raylib::GetGesturePinchVector" => Raylib_GetGesturePinchVector,
    "GetGesturePinchVector" => Raylib_GetGesturePinchVector,
    "raylib::GetGesturePinchAngle" => Raylib_GetGesturePinchAngle,
    "GetGesturePinchAngle" => Raylib_GetGesturePinchAngle,
    "raylib::UpdateCamera" => Raylib_UpdateCamera,
    "UpdateCamera" => Raylib_UpdateCamera,
    "raylib::UpdateCameraPro" => Raylib_UpdateCameraPro,
    "UpdateCameraPro" => Raylib_UpdateCameraPro,
    "raylib::SetShapesTexture" => Raylib_SetShapesTexture,
    "SetShapesTexture" => Raylib_SetShapesTexture,
    "raylib::GetShapesTexture" => Raylib_GetShapesTexture,
    "GetShapesTexture" => Raylib_GetShapesTexture,
    "raylib::GetShapesTextureRectangle" => Raylib_GetShapesTextureRectangle,
    "GetShapesTextureRectangle" => Raylib_GetShapesTextureRectangle,
    "raylib::DrawPixel" => Raylib_DrawPixel,
    "DrawPixel" => Raylib_DrawPixel,
    "raylib::DrawPixelV" => Raylib_DrawPixelV,
    "DrawPixelV" => Raylib_DrawPixelV,
    "raylib::DrawLine" => Raylib_DrawLine,
    "DrawLine" => Raylib_DrawLine,
    "raylib::DrawLineV" => Raylib_DrawLineV,
    "DrawLineV" => Raylib_DrawLineV,
    "raylib::DrawLineEx" => Raylib_DrawLineEx,
    "DrawLineEx" => Raylib_DrawLineEx,
    "raylib::DrawLineStrip" => Raylib_DrawLineStrip,
    "DrawLineStrip" => Raylib_DrawLineStrip,
    "raylib::DrawLineBezier" => Raylib_DrawLineBezier,
    "DrawLineBezier" => Raylib_DrawLineBezier,
    "raylib::DrawLineDashed" => Raylib_DrawLineDashed,
    "DrawLineDashed" => Raylib_DrawLineDashed,
    "raylib::DrawTriangle" => Raylib_DrawTriangle,
    "DrawTriangle" => Raylib_DrawTriangle,
    "raylib::DrawTriangleGradient" => Raylib_DrawTriangleGradient,
    "DrawTriangleGradient" => Raylib_DrawTriangleGradient,
    "raylib::DrawTriangleLines" => Raylib_DrawTriangleLines,
    "DrawTriangleLines" => Raylib_DrawTriangleLines,
    "raylib::DrawTriangleLinesEx" => Raylib_DrawTriangleLinesEx,
    "DrawTriangleLinesEx" => Raylib_DrawTriangleLinesEx,
    "raylib::DrawTriangleFan" => Raylib_DrawTriangleFan,
    "DrawTriangleFan" => Raylib_DrawTriangleFan,
    "raylib::DrawTriangleStrip" => Raylib_DrawTriangleStrip,
    "DrawTriangleStrip" => Raylib_DrawTriangleStrip,
    "raylib::DrawRectangle" => Raylib_DrawRectangle,
    "DrawRectangle" => Raylib_DrawRectangle,
    "raylib::DrawRectangleV" => Raylib_DrawRectangleV,
    "DrawRectangleV" => Raylib_DrawRectangleV,
    "raylib::DrawRectangleRec" => Raylib_DrawRectangleRec,
    "DrawRectangleRec" => Raylib_DrawRectangleRec,
    "raylib::DrawRectanglePro" => Raylib_DrawRectanglePro,
    "DrawRectanglePro" => Raylib_DrawRectanglePro,
    "raylib::DrawRectangleGradientV" => Raylib_DrawRectangleGradientV,
    "DrawRectangleGradientV" => Raylib_DrawRectangleGradientV,
    "raylib::DrawRectangleGradientH" => Raylib_DrawRectangleGradientH,
    "DrawRectangleGradientH" => Raylib_DrawRectangleGradientH,
    "raylib::DrawRectangleGradientEx" => Raylib_DrawRectangleGradientEx,
    "DrawRectangleGradientEx" => Raylib_DrawRectangleGradientEx,
    "raylib::DrawRectangleLines" => Raylib_DrawRectangleLines,
    "DrawRectangleLines" => Raylib_DrawRectangleLines,
    "raylib::DrawRectangleLinesEx" => Raylib_DrawRectangleLinesEx,
    "DrawRectangleLinesEx" => Raylib_DrawRectangleLinesEx,
    "raylib::DrawRectangleRounded" => Raylib_DrawRectangleRounded,
    "DrawRectangleRounded" => Raylib_DrawRectangleRounded,
    "raylib::DrawRectangleRoundedLines" => Raylib_DrawRectangleRoundedLines,
    "DrawRectangleRoundedLines" => Raylib_DrawRectangleRoundedLines,
    "raylib::DrawRectangleRoundedLinesEx" => Raylib_DrawRectangleRoundedLinesEx,
    "DrawRectangleRoundedLinesEx" => Raylib_DrawRectangleRoundedLinesEx,
    "raylib::DrawPoly" => Raylib_DrawPoly,
    "DrawPoly" => Raylib_DrawPoly,
    "raylib::DrawPolyLines" => Raylib_DrawPolyLines,
    "DrawPolyLines" => Raylib_DrawPolyLines,
    "raylib::DrawPolyLinesEx" => Raylib_DrawPolyLinesEx,
    "DrawPolyLinesEx" => Raylib_DrawPolyLinesEx,
    "raylib::DrawCircle" => Raylib_DrawCircle,
    "DrawCircle" => Raylib_DrawCircle,
    "raylib::DrawCircleV" => Raylib_DrawCircleV,
    "DrawCircleV" => Raylib_DrawCircleV,
    "raylib::DrawCircleGradient" => Raylib_DrawCircleGradient,
    "DrawCircleGradient" => Raylib_DrawCircleGradient,
    "raylib::DrawCircleSector" => Raylib_DrawCircleSector,
    "DrawCircleSector" => Raylib_DrawCircleSector,
    "raylib::DrawCircleSectorLines" => Raylib_DrawCircleSectorLines,
    "DrawCircleSectorLines" => Raylib_DrawCircleSectorLines,
    "raylib::DrawCircleSectorLinesEx" => Raylib_DrawCircleSectorLinesEx,
    "DrawCircleSectorLinesEx" => Raylib_DrawCircleSectorLinesEx,
    "raylib::DrawCircleLines" => Raylib_DrawCircleLines,
    "DrawCircleLines" => Raylib_DrawCircleLines,
    "raylib::DrawCircleLinesV" => Raylib_DrawCircleLinesV,
    "DrawCircleLinesV" => Raylib_DrawCircleLinesV,
    "raylib::DrawCircleLinesEx" => Raylib_DrawCircleLinesEx,
    "DrawCircleLinesEx" => Raylib_DrawCircleLinesEx,
    "raylib::DrawEllipse" => Raylib_DrawEllipse,
    "DrawEllipse" => Raylib_DrawEllipse,
    "raylib::DrawEllipseV" => Raylib_DrawEllipseV,
    "DrawEllipseV" => Raylib_DrawEllipseV,
    "raylib::DrawEllipseLines" => Raylib_DrawEllipseLines,
    "DrawEllipseLines" => Raylib_DrawEllipseLines,
    "raylib::DrawEllipseLinesV" => Raylib_DrawEllipseLinesV,
    "DrawEllipseLinesV" => Raylib_DrawEllipseLinesV,
    "raylib::DrawEllipseLinesEx" => Raylib_DrawEllipseLinesEx,
    "DrawEllipseLinesEx" => Raylib_DrawEllipseLinesEx,
    "raylib::DrawRing" => Raylib_DrawRing,
    "DrawRing" => Raylib_DrawRing,
    "raylib::DrawRingLines" => Raylib_DrawRingLines,
    "DrawRingLines" => Raylib_DrawRingLines,
    "raylib::DrawRingLinesEx" => Raylib_DrawRingLinesEx,
    "DrawRingLinesEx" => Raylib_DrawRingLinesEx,
    "raylib::DrawSplineLinear" => Raylib_DrawSplineLinear,
    "DrawSplineLinear" => Raylib_DrawSplineLinear,
    "raylib::DrawSplineBasis" => Raylib_DrawSplineBasis,
    "DrawSplineBasis" => Raylib_DrawSplineBasis,
    "raylib::DrawSplineCatmullRom" => Raylib_DrawSplineCatmullRom,
    "DrawSplineCatmullRom" => Raylib_DrawSplineCatmullRom,
    "raylib::DrawSplineBezierQuadratic" => Raylib_DrawSplineBezierQuadratic,
    "DrawSplineBezierQuadratic" => Raylib_DrawSplineBezierQuadratic,
    "raylib::DrawSplineBezierCubic" => Raylib_DrawSplineBezierCubic,
    "DrawSplineBezierCubic" => Raylib_DrawSplineBezierCubic,
    "raylib::DrawSplineSegmentLinear" => Raylib_DrawSplineSegmentLinear,
    "DrawSplineSegmentLinear" => Raylib_DrawSplineSegmentLinear,
    "raylib::DrawSplineSegmentBasis" => Raylib_DrawSplineSegmentBasis,
    "DrawSplineSegmentBasis" => Raylib_DrawSplineSegmentBasis,
    "raylib::DrawSplineSegmentCatmullRom" => Raylib_DrawSplineSegmentCatmullRom,
    "DrawSplineSegmentCatmullRom" => Raylib_DrawSplineSegmentCatmullRom,
    "raylib::DrawSplineSegmentBezierQuadratic" => Raylib_DrawSplineSegmentBezierQuadratic,
    "DrawSplineSegmentBezierQuadratic" => Raylib_DrawSplineSegmentBezierQuadratic,
    "raylib::DrawSplineSegmentBezierCubic" => Raylib_DrawSplineSegmentBezierCubic,
    "DrawSplineSegmentBezierCubic" => Raylib_DrawSplineSegmentBezierCubic,
    "raylib::GetSplinePointLinear" => Raylib_GetSplinePointLinear,
    "GetSplinePointLinear" => Raylib_GetSplinePointLinear,
    "raylib::GetSplinePointBasis" => Raylib_GetSplinePointBasis,
    "GetSplinePointBasis" => Raylib_GetSplinePointBasis,
    "raylib::GetSplinePointCatmullRom" => Raylib_GetSplinePointCatmullRom,
    "GetSplinePointCatmullRom" => Raylib_GetSplinePointCatmullRom,
    "raylib::GetSplinePointBezierQuadratic" => Raylib_GetSplinePointBezierQuadratic,
    "GetSplinePointBezierQuadratic" => Raylib_GetSplinePointBezierQuadratic,
    "raylib::GetSplinePointBezierCubic" => Raylib_GetSplinePointBezierCubic,
    "GetSplinePointBezierCubic" => Raylib_GetSplinePointBezierCubic,
    "raylib::CheckCollisionRecs" => Raylib_CheckCollisionRecs,
    "CheckCollisionRecs" => Raylib_CheckCollisionRecs,
    "raylib::CheckCollisionCircles" => Raylib_CheckCollisionCircles,
    "CheckCollisionCircles" => Raylib_CheckCollisionCircles,
    "raylib::CheckCollisionCircleRec" => Raylib_CheckCollisionCircleRec,
    "CheckCollisionCircleRec" => Raylib_CheckCollisionCircleRec,
    "raylib::CheckCollisionCircleLine" => Raylib_CheckCollisionCircleLine,
    "CheckCollisionCircleLine" => Raylib_CheckCollisionCircleLine,
    "raylib::CheckCollisionPointRec" => Raylib_CheckCollisionPointRec,
    "CheckCollisionPointRec" => Raylib_CheckCollisionPointRec,
    "raylib::CheckCollisionPointCircle" => Raylib_CheckCollisionPointCircle,
    "CheckCollisionPointCircle" => Raylib_CheckCollisionPointCircle,
    "raylib::CheckCollisionPointTriangle" => Raylib_CheckCollisionPointTriangle,
    "CheckCollisionPointTriangle" => Raylib_CheckCollisionPointTriangle,
    "raylib::CheckCollisionPointLine" => Raylib_CheckCollisionPointLine,
    "CheckCollisionPointLine" => Raylib_CheckCollisionPointLine,
    "raylib::CheckCollisionPointPoly" => Raylib_CheckCollisionPointPoly,
    "CheckCollisionPointPoly" => Raylib_CheckCollisionPointPoly,
    "raylib::CheckCollisionLines" => Raylib_CheckCollisionLines,
    "CheckCollisionLines" => Raylib_CheckCollisionLines,
    "raylib::GetCollisionRec" => Raylib_GetCollisionRec,
    "GetCollisionRec" => Raylib_GetCollisionRec,
    "raylib::LoadImage" => Raylib_LoadImage,
    "LoadImage" => Raylib_LoadImage,
    "raylib::LoadImageRaw" => Raylib_LoadImageRaw,
    "LoadImageRaw" => Raylib_LoadImageRaw,
    "raylib::LoadImageAnim" => Raylib_LoadImageAnim,
    "LoadImageAnim" => Raylib_LoadImageAnim,
    "raylib::LoadImageAnimFromMemory" => Raylib_LoadImageAnimFromMemory,
    "LoadImageAnimFromMemory" => Raylib_LoadImageAnimFromMemory,
    "raylib::LoadImageFromMemory" => Raylib_LoadImageFromMemory,
    "LoadImageFromMemory" => Raylib_LoadImageFromMemory,
    "raylib::LoadImageFromTexture" => Raylib_LoadImageFromTexture,
    "LoadImageFromTexture" => Raylib_LoadImageFromTexture,
    "raylib::LoadImageFromScreen" => Raylib_LoadImageFromScreen,
    "LoadImageFromScreen" => Raylib_LoadImageFromScreen,
    "raylib::IsImageValid" => Raylib_IsImageValid,
    "IsImageValid" => Raylib_IsImageValid,
    "raylib::UnloadImage" => Raylib_UnloadImage,
    "UnloadImage" => Raylib_UnloadImage,
    "raylib::ExportImage" => Raylib_ExportImage,
    "ExportImage" => Raylib_ExportImage,
    "raylib::ExportImageToMemory" => Raylib_ExportImageToMemory,
    "ExportImageToMemory" => Raylib_ExportImageToMemory,
    "raylib::ExportImageAsCode" => Raylib_ExportImageAsCode,
    "ExportImageAsCode" => Raylib_ExportImageAsCode,
    "raylib::GenImageColor" => Raylib_GenImageColor,
    "GenImageColor" => Raylib_GenImageColor,
    "raylib::GenImageGradientLinear" => Raylib_GenImageGradientLinear,
    "GenImageGradientLinear" => Raylib_GenImageGradientLinear,
    "raylib::GenImageGradientRadial" => Raylib_GenImageGradientRadial,
    "GenImageGradientRadial" => Raylib_GenImageGradientRadial,
    "raylib::GenImageGradientSquare" => Raylib_GenImageGradientSquare,
    "GenImageGradientSquare" => Raylib_GenImageGradientSquare,
    "raylib::GenImageChecked" => Raylib_GenImageChecked,
    "GenImageChecked" => Raylib_GenImageChecked,
    "raylib::GenImageWhiteNoise" => Raylib_GenImageWhiteNoise,
    "GenImageWhiteNoise" => Raylib_GenImageWhiteNoise,
    "raylib::GenImagePerlinNoise" => Raylib_GenImagePerlinNoise,
    "GenImagePerlinNoise" => Raylib_GenImagePerlinNoise,
    "raylib::GenImageCellular" => Raylib_GenImageCellular,
    "GenImageCellular" => Raylib_GenImageCellular,
    "raylib::GenImageText" => Raylib_GenImageText,
    "GenImageText" => Raylib_GenImageText,
    "raylib::ImageCopy" => Raylib_ImageCopy,
    "ImageCopy" => Raylib_ImageCopy,
    "raylib::ImageFromImage" => Raylib_ImageFromImage,
    "ImageFromImage" => Raylib_ImageFromImage,
    "raylib::ImageFromChannel" => Raylib_ImageFromChannel,
    "ImageFromChannel" => Raylib_ImageFromChannel,
    "raylib::ImageText" => Raylib_ImageText,
    "ImageText" => Raylib_ImageText,
    "raylib::ImageTextEx" => Raylib_ImageTextEx,
    "ImageTextEx" => Raylib_ImageTextEx,
    "raylib::ImageFormat" => Raylib_ImageFormat,
    "ImageFormat" => Raylib_ImageFormat,
    "raylib::ImageToPOT" => Raylib_ImageToPOT,
    "ImageToPOT" => Raylib_ImageToPOT,
    "raylib::ImageCrop" => Raylib_ImageCrop,
    "ImageCrop" => Raylib_ImageCrop,
    "raylib::ImageAlphaCrop" => Raylib_ImageAlphaCrop,
    "ImageAlphaCrop" => Raylib_ImageAlphaCrop,
    "raylib::ImageAlphaClear" => Raylib_ImageAlphaClear,
    "ImageAlphaClear" => Raylib_ImageAlphaClear,
    "raylib::ImageAlphaMask" => Raylib_ImageAlphaMask,
    "ImageAlphaMask" => Raylib_ImageAlphaMask,
    "raylib::ImageAlphaPremultiply" => Raylib_ImageAlphaPremultiply,
    "ImageAlphaPremultiply" => Raylib_ImageAlphaPremultiply,
    "raylib::ImageBlurGaussian" => Raylib_ImageBlurGaussian,
    "ImageBlurGaussian" => Raylib_ImageBlurGaussian,
    "raylib::ImageKernelConvolution" => Raylib_ImageKernelConvolution,
    "ImageKernelConvolution" => Raylib_ImageKernelConvolution,
    "raylib::ImageResize" => Raylib_ImageResize,
    "ImageResize" => Raylib_ImageResize,
    "raylib::ImageResizeNN" => Raylib_ImageResizeNN,
    "ImageResizeNN" => Raylib_ImageResizeNN,
    "raylib::ImageResizeCanvas" => Raylib_ImageResizeCanvas,
    "ImageResizeCanvas" => Raylib_ImageResizeCanvas,
    "raylib::ImageMipmaps" => Raylib_ImageMipmaps,
    "ImageMipmaps" => Raylib_ImageMipmaps,
    "raylib::ImageDither" => Raylib_ImageDither,
    "ImageDither" => Raylib_ImageDither,
    "raylib::ImageFlipVertical" => Raylib_ImageFlipVertical,
    "ImageFlipVertical" => Raylib_ImageFlipVertical,
    "raylib::ImageFlipHorizontal" => Raylib_ImageFlipHorizontal,
    "ImageFlipHorizontal" => Raylib_ImageFlipHorizontal,
    "raylib::ImageRotate" => Raylib_ImageRotate,
    "ImageRotate" => Raylib_ImageRotate,
    "raylib::ImageRotateCW" => Raylib_ImageRotateCW,
    "ImageRotateCW" => Raylib_ImageRotateCW,
    "raylib::ImageRotateCCW" => Raylib_ImageRotateCCW,
    "ImageRotateCCW" => Raylib_ImageRotateCCW,
    "raylib::ImageColorTint" => Raylib_ImageColorTint,
    "ImageColorTint" => Raylib_ImageColorTint,
    "raylib::ImageColorInvert" => Raylib_ImageColorInvert,
    "ImageColorInvert" => Raylib_ImageColorInvert,
    "raylib::ImageColorGrayscale" => Raylib_ImageColorGrayscale,
    "ImageColorGrayscale" => Raylib_ImageColorGrayscale,
    "raylib::ImageColorContrast" => Raylib_ImageColorContrast,
    "ImageColorContrast" => Raylib_ImageColorContrast,
    "raylib::ImageColorBrightness" => Raylib_ImageColorBrightness,
    "ImageColorBrightness" => Raylib_ImageColorBrightness,
    "raylib::ImageColorReplace" => Raylib_ImageColorReplace,
    "ImageColorReplace" => Raylib_ImageColorReplace,
    "raylib::LoadImageColors" => Raylib_LoadImageColors,
    "LoadImageColors" => Raylib_LoadImageColors,
    "raylib::LoadImagePalette" => Raylib_LoadImagePalette,
    "LoadImagePalette" => Raylib_LoadImagePalette,
    "raylib::UnloadImageColors" => Raylib_UnloadImageColors,
    "UnloadImageColors" => Raylib_UnloadImageColors,
    "raylib::UnloadImagePalette" => Raylib_UnloadImagePalette,
    "UnloadImagePalette" => Raylib_UnloadImagePalette,
    "raylib::GetImageAlphaBorder" => Raylib_GetImageAlphaBorder,
    "GetImageAlphaBorder" => Raylib_GetImageAlphaBorder,
    "raylib::GetImageColor" => Raylib_GetImageColor,
    "GetImageColor" => Raylib_GetImageColor,
    "raylib::ImageClearBackground" => Raylib_ImageClearBackground,
    "ImageClearBackground" => Raylib_ImageClearBackground,
    "raylib::ImageDrawPixel" => Raylib_ImageDrawPixel,
    "ImageDrawPixel" => Raylib_ImageDrawPixel,
    "raylib::ImageDrawPixelV" => Raylib_ImageDrawPixelV,
    "ImageDrawPixelV" => Raylib_ImageDrawPixelV,
    "raylib::ImageDrawLine" => Raylib_ImageDrawLine,
    "ImageDrawLine" => Raylib_ImageDrawLine,
    "raylib::ImageDrawLineV" => Raylib_ImageDrawLineV,
    "ImageDrawLineV" => Raylib_ImageDrawLineV,
    "raylib::ImageDrawLineEx" => Raylib_ImageDrawLineEx,
    "ImageDrawLineEx" => Raylib_ImageDrawLineEx,
    "raylib::ImageDrawLineStrip" => Raylib_ImageDrawLineStrip,
    "ImageDrawLineStrip" => Raylib_ImageDrawLineStrip,
    "raylib::ImageDrawTriangle" => Raylib_ImageDrawTriangle,
    "ImageDrawTriangle" => Raylib_ImageDrawTriangle,
    "raylib::ImageDrawTriangleGradient" => Raylib_ImageDrawTriangleGradient,
    "ImageDrawTriangleGradient" => Raylib_ImageDrawTriangleGradient,
    "raylib::ImageDrawTriangleLines" => Raylib_ImageDrawTriangleLines,
    "ImageDrawTriangleLines" => Raylib_ImageDrawTriangleLines,
    "raylib::ImageDrawTriangleFan" => Raylib_ImageDrawTriangleFan,
    "ImageDrawTriangleFan" => Raylib_ImageDrawTriangleFan,
    "raylib::ImageDrawTriangleStrip" => Raylib_ImageDrawTriangleStrip,
    "ImageDrawTriangleStrip" => Raylib_ImageDrawTriangleStrip,
    "raylib::ImageDrawRectangle" => Raylib_ImageDrawRectangle,
    "ImageDrawRectangle" => Raylib_ImageDrawRectangle,
    "raylib::ImageDrawRectangleV" => Raylib_ImageDrawRectangleV,
    "ImageDrawRectangleV" => Raylib_ImageDrawRectangleV,
    "raylib::ImageDrawRectangleRec" => Raylib_ImageDrawRectangleRec,
    "ImageDrawRectangleRec" => Raylib_ImageDrawRectangleRec,
    "raylib::ImageDrawRectanglePro" => Raylib_ImageDrawRectanglePro,
    "ImageDrawRectanglePro" => Raylib_ImageDrawRectanglePro,
    "raylib::ImageDrawRectangleLines" => Raylib_ImageDrawRectangleLines,
    "ImageDrawRectangleLines" => Raylib_ImageDrawRectangleLines,
    "raylib::ImageDrawRectangleLinesEx" => Raylib_ImageDrawRectangleLinesEx,
    "ImageDrawRectangleLinesEx" => Raylib_ImageDrawRectangleLinesEx,
    "raylib::ImageDrawRectangleGradientEx" => Raylib_ImageDrawRectangleGradientEx,
    "ImageDrawRectangleGradientEx" => Raylib_ImageDrawRectangleGradientEx,
    "raylib::ImageDrawCircle" => Raylib_ImageDrawCircle,
    "ImageDrawCircle" => Raylib_ImageDrawCircle,
    "raylib::ImageDrawCircleV" => Raylib_ImageDrawCircleV,
    "ImageDrawCircleV" => Raylib_ImageDrawCircleV,
    "raylib::ImageDrawCircleLines" => Raylib_ImageDrawCircleLines,
    "ImageDrawCircleLines" => Raylib_ImageDrawCircleLines,
    "raylib::ImageDrawCircleLinesV" => Raylib_ImageDrawCircleLinesV,
    "ImageDrawCircleLinesV" => Raylib_ImageDrawCircleLinesV,
    "raylib::ImageDrawCircleGradient" => Raylib_ImageDrawCircleGradient,
    "ImageDrawCircleGradient" => Raylib_ImageDrawCircleGradient,
    "raylib::ImageDrawImage" => Raylib_ImageDrawImage,
    "ImageDrawImage" => Raylib_ImageDrawImage,
    "raylib::ImageDrawImageEx" => Raylib_ImageDrawImageEx,
    "ImageDrawImageEx" => Raylib_ImageDrawImageEx,
    "raylib::ImageDrawImageRec" => Raylib_ImageDrawImageRec,
    "ImageDrawImageRec" => Raylib_ImageDrawImageRec,
    "raylib::ImageDrawImagePro" => Raylib_ImageDrawImagePro,
    "ImageDrawImagePro" => Raylib_ImageDrawImagePro,
    "raylib::ImageDrawText" => Raylib_ImageDrawText,
    "ImageDrawText" => Raylib_ImageDrawText,
    "raylib::ImageDrawTextEx" => Raylib_ImageDrawTextEx,
    "ImageDrawTextEx" => Raylib_ImageDrawTextEx,
    "raylib::ImageDrawTextPro" => Raylib_ImageDrawTextPro,
    "ImageDrawTextPro" => Raylib_ImageDrawTextPro,
    "raylib::LoadTexture" => Raylib_LoadTexture,
    "LoadTexture" => Raylib_LoadTexture,
    "raylib::LoadTextureFromImage" => Raylib_LoadTextureFromImage,
    "LoadTextureFromImage" => Raylib_LoadTextureFromImage,
    "raylib::LoadTextureCubemap" => Raylib_LoadTextureCubemap,
    "LoadTextureCubemap" => Raylib_LoadTextureCubemap,
    "raylib::LoadRenderTexture" => Raylib_LoadRenderTexture,
    "LoadRenderTexture" => Raylib_LoadRenderTexture,
    "raylib::LoadRenderTextureEx" => Raylib_LoadRenderTextureEx,
    "LoadRenderTextureEx" => Raylib_LoadRenderTextureEx,
    "raylib::IsTextureValid" => Raylib_IsTextureValid,
    "IsTextureValid" => Raylib_IsTextureValid,
    "raylib::UnloadTexture" => Raylib_UnloadTexture,
    "UnloadTexture" => Raylib_UnloadTexture,
    "raylib::IsRenderTextureValid" => Raylib_IsRenderTextureValid,
    "IsRenderTextureValid" => Raylib_IsRenderTextureValid,
    "raylib::UnloadRenderTexture" => Raylib_UnloadRenderTexture,
    "UnloadRenderTexture" => Raylib_UnloadRenderTexture,
    "raylib::UpdateTexture" => Raylib_UpdateTexture,
    "UpdateTexture" => Raylib_UpdateTexture,
    "raylib::UpdateTextureRec" => Raylib_UpdateTextureRec,
    "UpdateTextureRec" => Raylib_UpdateTextureRec,
    "raylib::GenTextureMipmaps" => Raylib_GenTextureMipmaps,
    "GenTextureMipmaps" => Raylib_GenTextureMipmaps,
    "raylib::SetTextureFilter" => Raylib_SetTextureFilter,
    "SetTextureFilter" => Raylib_SetTextureFilter,
    "raylib::SetTextureWrap" => Raylib_SetTextureWrap,
    "SetTextureWrap" => Raylib_SetTextureWrap,
    "raylib::DrawTexture" => Raylib_DrawTexture,
    "DrawTexture" => Raylib_DrawTexture,
    "raylib::DrawTextureV" => Raylib_DrawTextureV,
    "DrawTextureV" => Raylib_DrawTextureV,
    "raylib::DrawTextureEx" => Raylib_DrawTextureEx,
    "DrawTextureEx" => Raylib_DrawTextureEx,
    "raylib::DrawTextureRec" => Raylib_DrawTextureRec,
    "DrawTextureRec" => Raylib_DrawTextureRec,
    "raylib::DrawTexturePro" => Raylib_DrawTexturePro,
    "DrawTexturePro" => Raylib_DrawTexturePro,
    "raylib::DrawTextureNPatch" => Raylib_DrawTextureNPatch,
    "DrawTextureNPatch" => Raylib_DrawTextureNPatch,
    "raylib::ColorIsEqual" => Raylib_ColorIsEqual,
    "ColorIsEqual" => Raylib_ColorIsEqual,
    "raylib::Fade" => Raylib_Fade,
    "Fade" => Raylib_Fade,
    "raylib::ColorToInt" => Raylib_ColorToInt,
    "ColorToInt" => Raylib_ColorToInt,
    "raylib::ColorNormalize" => Raylib_ColorNormalize,
    "ColorNormalize" => Raylib_ColorNormalize,
    "raylib::ColorFromNormalized" => Raylib_ColorFromNormalized,
    "ColorFromNormalized" => Raylib_ColorFromNormalized,
    "raylib::ColorToHSV" => Raylib_ColorToHSV,
    "ColorToHSV" => Raylib_ColorToHSV,
    "raylib::ColorFromHSV" => Raylib_ColorFromHSV,
    "ColorFromHSV" => Raylib_ColorFromHSV,
    "raylib::ColorTint" => Raylib_ColorTint,
    "ColorTint" => Raylib_ColorTint,
    "raylib::ColorBrightness" => Raylib_ColorBrightness,
    "ColorBrightness" => Raylib_ColorBrightness,
    "raylib::ColorContrast" => Raylib_ColorContrast,
    "ColorContrast" => Raylib_ColorContrast,
    "raylib::ColorAlpha" => Raylib_ColorAlpha,
    "ColorAlpha" => Raylib_ColorAlpha,
    "raylib::ColorAlphaBlend" => Raylib_ColorAlphaBlend,
    "ColorAlphaBlend" => Raylib_ColorAlphaBlend,
    "raylib::ColorLerp" => Raylib_ColorLerp,
    "ColorLerp" => Raylib_ColorLerp,
    "raylib::GetColor" => Raylib_GetColor,
    "GetColor" => Raylib_GetColor,
    "raylib::GetPixelColor" => Raylib_GetPixelColor,
    "GetPixelColor" => Raylib_GetPixelColor,
    "raylib::SetPixelColor" => Raylib_SetPixelColor,
    "SetPixelColor" => Raylib_SetPixelColor,
    "raylib::GetPixelDataSize" => Raylib_GetPixelDataSize,
    "GetPixelDataSize" => Raylib_GetPixelDataSize,
    "raylib::GetFontDefault" => Raylib_GetFontDefault,
    "GetFontDefault" => Raylib_GetFontDefault,
    "raylib::LoadFont" => Raylib_LoadFont,
    "LoadFont" => Raylib_LoadFont,
    "raylib::LoadFontEx" => Raylib_LoadFontEx,
    "LoadFontEx" => Raylib_LoadFontEx,
    "raylib::LoadFontFromImage" => Raylib_LoadFontFromImage,
    "LoadFontFromImage" => Raylib_LoadFontFromImage,
    "raylib::LoadFontFromMemory" => Raylib_LoadFontFromMemory,
    "LoadFontFromMemory" => Raylib_LoadFontFromMemory,
    "raylib::IsFontValid" => Raylib_IsFontValid,
    "IsFontValid" => Raylib_IsFontValid,
    "raylib::LoadFontData" => Raylib_LoadFontData,
    "LoadFontData" => Raylib_LoadFontData,
    "raylib::GenImageFontAtlas" => Raylib_GenImageFontAtlas,
    "GenImageFontAtlas" => Raylib_GenImageFontAtlas,
    "raylib::UnloadFontData" => Raylib_UnloadFontData,
    "UnloadFontData" => Raylib_UnloadFontData,
    "raylib::UnloadFont" => Raylib_UnloadFont,
    "UnloadFont" => Raylib_UnloadFont,
    "raylib::ExportFontAsCode" => Raylib_ExportFontAsCode,
    "ExportFontAsCode" => Raylib_ExportFontAsCode,
    "raylib::DrawFPS" => Raylib_DrawFPS,
    "DrawFPS" => Raylib_DrawFPS,
    "raylib::DrawText" => Raylib_DrawText,
    "DrawText" => Raylib_DrawText,
    "raylib::DrawTextEx" => Raylib_DrawTextEx,
    "DrawTextEx" => Raylib_DrawTextEx,
    "raylib::DrawTextPro" => Raylib_DrawTextPro,
    "DrawTextPro" => Raylib_DrawTextPro,
    "raylib::DrawTextCodepoint" => Raylib_DrawTextCodepoint,
    "DrawTextCodepoint" => Raylib_DrawTextCodepoint,
    "raylib::DrawTextCodepoints" => Raylib_DrawTextCodepoints,
    "DrawTextCodepoints" => Raylib_DrawTextCodepoints,
    "raylib::SetTextLineSpacing" => Raylib_SetTextLineSpacing,
    "SetTextLineSpacing" => Raylib_SetTextLineSpacing,
    "raylib::MeasureText" => Raylib_MeasureText,
    "MeasureText" => Raylib_MeasureText,
    "raylib::MeasureTextEx" => Raylib_MeasureTextEx,
    "MeasureTextEx" => Raylib_MeasureTextEx,
    "raylib::MeasureTextCodepoints" => Raylib_MeasureTextCodepoints,
    "MeasureTextCodepoints" => Raylib_MeasureTextCodepoints,
    "raylib::GetGlyphIndex" => Raylib_GetGlyphIndex,
    "GetGlyphIndex" => Raylib_GetGlyphIndex,
    "raylib::GetGlyphInfo" => Raylib_GetGlyphInfo,
    "GetGlyphInfo" => Raylib_GetGlyphInfo,
    "raylib::GetGlyphAtlasRec" => Raylib_GetGlyphAtlasRec,
    "GetGlyphAtlasRec" => Raylib_GetGlyphAtlasRec,
    "raylib::LoadUTF8" => Raylib_LoadUTF8,
    "LoadUTF8" => Raylib_LoadUTF8,
    "raylib::UnloadUTF8" => Raylib_UnloadUTF8,
    "UnloadUTF8" => Raylib_UnloadUTF8,
    "raylib::LoadCodepoints" => Raylib_LoadCodepoints,
    "LoadCodepoints" => Raylib_LoadCodepoints,
    "raylib::UnloadCodepoints" => Raylib_UnloadCodepoints,
    "UnloadCodepoints" => Raylib_UnloadCodepoints,
    "raylib::GetCodepointCount" => Raylib_GetCodepointCount,
    "GetCodepointCount" => Raylib_GetCodepointCount,
    "raylib::GetCodepoint" => Raylib_GetCodepoint,
    "GetCodepoint" => Raylib_GetCodepoint,
    "raylib::GetCodepointNext" => Raylib_GetCodepointNext,
    "GetCodepointNext" => Raylib_GetCodepointNext,
    "raylib::GetCodepointPrevious" => Raylib_GetCodepointPrevious,
    "GetCodepointPrevious" => Raylib_GetCodepointPrevious,
    "raylib::CodepointToUTF8" => Raylib_CodepointToUTF8,
    "CodepointToUTF8" => Raylib_CodepointToUTF8,
    "raylib::LoadTextLines" => Raylib_LoadTextLines,
    "LoadTextLines" => Raylib_LoadTextLines,
    "raylib::UnloadTextLines" => Raylib_UnloadTextLines,
    "UnloadTextLines" => Raylib_UnloadTextLines,
    "raylib::TextCopy" => Raylib_TextCopy,
    "TextCopy" => Raylib_TextCopy,
    "raylib::TextIsEqual" => Raylib_TextIsEqual,
    "TextIsEqual" => Raylib_TextIsEqual,
    "raylib::TextLength" => Raylib_TextLength,
    "TextLength" => Raylib_TextLength,
    "raylib::TextFormat" => Raylib_TextFormat,
    "TextFormat" => Raylib_TextFormat,
    "raylib::TextSubtext" => Raylib_TextSubtext,
    "TextSubtext" => Raylib_TextSubtext,
    "raylib::TextRemoveSpaces" => Raylib_TextRemoveSpaces,
    "TextRemoveSpaces" => Raylib_TextRemoveSpaces,
    "raylib::GetTextBetween" => Raylib_GetTextBetween,
    "GetTextBetween" => Raylib_GetTextBetween,
    "raylib::TextReplace" => Raylib_TextReplace,
    "TextReplace" => Raylib_TextReplace,
    "raylib::TextReplaceAlloc" => Raylib_TextReplaceAlloc,
    "TextReplaceAlloc" => Raylib_TextReplaceAlloc,
    "raylib::TextReplaceBetween" => Raylib_TextReplaceBetween,
    "TextReplaceBetween" => Raylib_TextReplaceBetween,
    "raylib::TextReplaceBetweenAlloc" => Raylib_TextReplaceBetweenAlloc,
    "TextReplaceBetweenAlloc" => Raylib_TextReplaceBetweenAlloc,
    "raylib::TextInsert" => Raylib_TextInsert,
    "TextInsert" => Raylib_TextInsert,
    "raylib::TextInsertAlloc" => Raylib_TextInsertAlloc,
    "TextInsertAlloc" => Raylib_TextInsertAlloc,
    "raylib::TextJoin" => Raylib_TextJoin,
    "TextJoin" => Raylib_TextJoin,
    "raylib::TextSplit" => Raylib_TextSplit,
    "TextSplit" => Raylib_TextSplit,
    "raylib::TextAppend" => Raylib_TextAppend,
    "TextAppend" => Raylib_TextAppend,
    "raylib::TextFindIndex" => Raylib_TextFindIndex,
    "TextFindIndex" => Raylib_TextFindIndex,
    "raylib::TextToUpper" => Raylib_TextToUpper,
    "TextToUpper" => Raylib_TextToUpper,
    "raylib::TextToLower" => Raylib_TextToLower,
    "TextToLower" => Raylib_TextToLower,
    "raylib::TextToPascal" => Raylib_TextToPascal,
    "TextToPascal" => Raylib_TextToPascal,
    "raylib::TextToSnake" => Raylib_TextToSnake,
    "TextToSnake" => Raylib_TextToSnake,
    "raylib::TextToCamel" => Raylib_TextToCamel,
    "TextToCamel" => Raylib_TextToCamel,
    "raylib::TextToInteger" => Raylib_TextToInteger,
    "TextToInteger" => Raylib_TextToInteger,
    "raylib::TextToFloat" => Raylib_TextToFloat,
    "TextToFloat" => Raylib_TextToFloat,
    "raylib::DrawLine3D" => Raylib_DrawLine3D,
    "DrawLine3D" => Raylib_DrawLine3D,
    "raylib::DrawPoint3D" => Raylib_DrawPoint3D,
    "DrawPoint3D" => Raylib_DrawPoint3D,
    "raylib::DrawCircle3D" => Raylib_DrawCircle3D,
    "DrawCircle3D" => Raylib_DrawCircle3D,
    "raylib::DrawTriangle3D" => Raylib_DrawTriangle3D,
    "DrawTriangle3D" => Raylib_DrawTriangle3D,
    "raylib::DrawTriangleStrip3D" => Raylib_DrawTriangleStrip3D,
    "DrawTriangleStrip3D" => Raylib_DrawTriangleStrip3D,
    "raylib::DrawCube" => Raylib_DrawCube,
    "DrawCube" => Raylib_DrawCube,
    "raylib::DrawCubeV" => Raylib_DrawCubeV,
    "DrawCubeV" => Raylib_DrawCubeV,
    "raylib::DrawCubeWires" => Raylib_DrawCubeWires,
    "DrawCubeWires" => Raylib_DrawCubeWires,
    "raylib::DrawCubeWiresV" => Raylib_DrawCubeWiresV,
    "DrawCubeWiresV" => Raylib_DrawCubeWiresV,
    "raylib::DrawSphere" => Raylib_DrawSphere,
    "DrawSphere" => Raylib_DrawSphere,
    "raylib::DrawSphereEx" => Raylib_DrawSphereEx,
    "DrawSphereEx" => Raylib_DrawSphereEx,
    "raylib::DrawSphereWires" => Raylib_DrawSphereWires,
    "DrawSphereWires" => Raylib_DrawSphereWires,
    "raylib::DrawCylinder" => Raylib_DrawCylinder,
    "DrawCylinder" => Raylib_DrawCylinder,
    "raylib::DrawCylinderEx" => Raylib_DrawCylinderEx,
    "DrawCylinderEx" => Raylib_DrawCylinderEx,
    "raylib::DrawCylinderWires" => Raylib_DrawCylinderWires,
    "DrawCylinderWires" => Raylib_DrawCylinderWires,
    "raylib::DrawCylinderWiresEx" => Raylib_DrawCylinderWiresEx,
    "DrawCylinderWiresEx" => Raylib_DrawCylinderWiresEx,
    "raylib::DrawCapsule" => Raylib_DrawCapsule,
    "DrawCapsule" => Raylib_DrawCapsule,
    "raylib::DrawCapsuleWires" => Raylib_DrawCapsuleWires,
    "DrawCapsuleWires" => Raylib_DrawCapsuleWires,
    "raylib::DrawPlane" => Raylib_DrawPlane,
    "DrawPlane" => Raylib_DrawPlane,
    "raylib::DrawRay" => Raylib_DrawRay,
    "DrawRay" => Raylib_DrawRay,
    "raylib::DrawGrid" => Raylib_DrawGrid,
    "DrawGrid" => Raylib_DrawGrid,
    "raylib::LoadModel" => Raylib_LoadModel,
    "LoadModel" => Raylib_LoadModel,
    "raylib::LoadModelFromMesh" => Raylib_LoadModelFromMesh,
    "LoadModelFromMesh" => Raylib_LoadModelFromMesh,
    "raylib::IsModelValid" => Raylib_IsModelValid,
    "IsModelValid" => Raylib_IsModelValid,
    "raylib::UnloadModel" => Raylib_UnloadModel,
    "UnloadModel" => Raylib_UnloadModel,
    "raylib::GetModelBoundingBox" => Raylib_GetModelBoundingBox,
    "GetModelBoundingBox" => Raylib_GetModelBoundingBox,
    "raylib::DrawModel" => Raylib_DrawModel,
    "DrawModel" => Raylib_DrawModel,
    "raylib::DrawModelEx" => Raylib_DrawModelEx,
    "DrawModelEx" => Raylib_DrawModelEx,
    "raylib::DrawModelWires" => Raylib_DrawModelWires,
    "DrawModelWires" => Raylib_DrawModelWires,
    "raylib::DrawModelWiresEx" => Raylib_DrawModelWiresEx,
    "DrawModelWiresEx" => Raylib_DrawModelWiresEx,
    "raylib::DrawBoundingBox" => Raylib_DrawBoundingBox,
    "DrawBoundingBox" => Raylib_DrawBoundingBox,
    "raylib::DrawBillboard" => Raylib_DrawBillboard,
    "DrawBillboard" => Raylib_DrawBillboard,
    "raylib::DrawBillboardRec" => Raylib_DrawBillboardRec,
    "DrawBillboardRec" => Raylib_DrawBillboardRec,
    "raylib::DrawBillboardPro" => Raylib_DrawBillboardPro,
    "DrawBillboardPro" => Raylib_DrawBillboardPro,
    "raylib::UploadMesh" => Raylib_UploadMesh,
    "UploadMesh" => Raylib_UploadMesh,
    "raylib::UpdateMeshBuffer" => Raylib_UpdateMeshBuffer,
    "UpdateMeshBuffer" => Raylib_UpdateMeshBuffer,
    "raylib::UnloadMesh" => Raylib_UnloadMesh,
    "UnloadMesh" => Raylib_UnloadMesh,
    "raylib::DrawMesh" => Raylib_DrawMesh,
    "DrawMesh" => Raylib_DrawMesh,
    "raylib::DrawMeshInstanced" => Raylib_DrawMeshInstanced,
    "DrawMeshInstanced" => Raylib_DrawMeshInstanced,
    "raylib::GetMeshBoundingBox" => Raylib_GetMeshBoundingBox,
    "GetMeshBoundingBox" => Raylib_GetMeshBoundingBox,
    "raylib::GenMeshTangents" => Raylib_GenMeshTangents,
    "GenMeshTangents" => Raylib_GenMeshTangents,
    "raylib::ExportMesh" => Raylib_ExportMesh,
    "ExportMesh" => Raylib_ExportMesh,
    "raylib::ExportMeshAsCode" => Raylib_ExportMeshAsCode,
    "ExportMeshAsCode" => Raylib_ExportMeshAsCode,
    "raylib::GenMeshPoly" => Raylib_GenMeshPoly,
    "GenMeshPoly" => Raylib_GenMeshPoly,
    "raylib::GenMeshPlane" => Raylib_GenMeshPlane,
    "GenMeshPlane" => Raylib_GenMeshPlane,
    "raylib::GenMeshCube" => Raylib_GenMeshCube,
    "GenMeshCube" => Raylib_GenMeshCube,
    "raylib::GenMeshSphere" => Raylib_GenMeshSphere,
    "GenMeshSphere" => Raylib_GenMeshSphere,
    "raylib::GenMeshHemiSphere" => Raylib_GenMeshHemiSphere,
    "GenMeshHemiSphere" => Raylib_GenMeshHemiSphere,
    "raylib::GenMeshCylinder" => Raylib_GenMeshCylinder,
    "GenMeshCylinder" => Raylib_GenMeshCylinder,
    "raylib::GenMeshCone" => Raylib_GenMeshCone,
    "GenMeshCone" => Raylib_GenMeshCone,
    "raylib::GenMeshTorus" => Raylib_GenMeshTorus,
    "GenMeshTorus" => Raylib_GenMeshTorus,
    "raylib::GenMeshKnot" => Raylib_GenMeshKnot,
    "GenMeshKnot" => Raylib_GenMeshKnot,
    "raylib::GenMeshHeightmap" => Raylib_GenMeshHeightmap,
    "GenMeshHeightmap" => Raylib_GenMeshHeightmap,
    "raylib::GenMeshCubicmap" => Raylib_GenMeshCubicmap,
    "GenMeshCubicmap" => Raylib_GenMeshCubicmap,
    "raylib::LoadMaterials" => Raylib_LoadMaterials,
    "LoadMaterials" => Raylib_LoadMaterials,
    "raylib::LoadMaterialDefault" => Raylib_LoadMaterialDefault,
    "LoadMaterialDefault" => Raylib_LoadMaterialDefault,
    "raylib::IsMaterialValid" => Raylib_IsMaterialValid,
    "IsMaterialValid" => Raylib_IsMaterialValid,
    "raylib::UnloadMaterial" => Raylib_UnloadMaterial,
    "UnloadMaterial" => Raylib_UnloadMaterial,
    "raylib::SetMaterialTexture" => Raylib_SetMaterialTexture,
    "SetMaterialTexture" => Raylib_SetMaterialTexture,
    "raylib::SetModelMeshMaterial" => Raylib_SetModelMeshMaterial,
    "SetModelMeshMaterial" => Raylib_SetModelMeshMaterial,
    "raylib::LoadModelAnimations" => Raylib_LoadModelAnimations,
    "LoadModelAnimations" => Raylib_LoadModelAnimations,
    "raylib::UpdateModelAnimation" => Raylib_UpdateModelAnimation,
    "UpdateModelAnimation" => Raylib_UpdateModelAnimation,
    "raylib::UpdateModelAnimationEx" => Raylib_UpdateModelAnimationEx,
    "UpdateModelAnimationEx" => Raylib_UpdateModelAnimationEx,
    "raylib::UnloadModelAnimations" => Raylib_UnloadModelAnimations,
    "UnloadModelAnimations" => Raylib_UnloadModelAnimations,
    "raylib::IsModelAnimationValid" => Raylib_IsModelAnimationValid,
    "IsModelAnimationValid" => Raylib_IsModelAnimationValid,
    "raylib::CheckCollisionSpheres" => Raylib_CheckCollisionSpheres,
    "CheckCollisionSpheres" => Raylib_CheckCollisionSpheres,
    "raylib::CheckCollisionBoxes" => Raylib_CheckCollisionBoxes,
    "CheckCollisionBoxes" => Raylib_CheckCollisionBoxes,
    "raylib::CheckCollisionBoxSphere" => Raylib_CheckCollisionBoxSphere,
    "CheckCollisionBoxSphere" => Raylib_CheckCollisionBoxSphere,
    "raylib::GetRayCollisionSphere" => Raylib_GetRayCollisionSphere,
    "GetRayCollisionSphere" => Raylib_GetRayCollisionSphere,
    "raylib::GetRayCollisionBox" => Raylib_GetRayCollisionBox,
    "GetRayCollisionBox" => Raylib_GetRayCollisionBox,
    "raylib::GetRayCollisionMesh" => Raylib_GetRayCollisionMesh,
    "GetRayCollisionMesh" => Raylib_GetRayCollisionMesh,
    "raylib::GetRayCollisionTriangle" => Raylib_GetRayCollisionTriangle,
    "GetRayCollisionTriangle" => Raylib_GetRayCollisionTriangle,
    "raylib::GetRayCollisionQuad" => Raylib_GetRayCollisionQuad,
    "GetRayCollisionQuad" => Raylib_GetRayCollisionQuad,
    "raylib::InitAudioDevice" => Raylib_InitAudioDevice,
    "InitAudioDevice" => Raylib_InitAudioDevice,
    "raylib::CloseAudioDevice" => Raylib_CloseAudioDevice,
    "CloseAudioDevice" => Raylib_CloseAudioDevice,
    "raylib::IsAudioDeviceReady" => Raylib_IsAudioDeviceReady,
    "IsAudioDeviceReady" => Raylib_IsAudioDeviceReady,
    "raylib::SetMasterVolume" => Raylib_SetMasterVolume,
    "SetMasterVolume" => Raylib_SetMasterVolume,
    "raylib::GetMasterVolume" => Raylib_GetMasterVolume,
    "GetMasterVolume" => Raylib_GetMasterVolume,
    "raylib::LoadWave" => Raylib_LoadWave,
    "LoadWave" => Raylib_LoadWave,
    "raylib::LoadWaveFromMemory" => Raylib_LoadWaveFromMemory,
    "LoadWaveFromMemory" => Raylib_LoadWaveFromMemory,
    "raylib::IsWaveValid" => Raylib_IsWaveValid,
    "IsWaveValid" => Raylib_IsWaveValid,
    "raylib::LoadSound" => Raylib_LoadSound,
    "LoadSound" => Raylib_LoadSound,
    "raylib::LoadSoundFromWave" => Raylib_LoadSoundFromWave,
    "LoadSoundFromWave" => Raylib_LoadSoundFromWave,
    "raylib::LoadSoundAlias" => Raylib_LoadSoundAlias,
    "LoadSoundAlias" => Raylib_LoadSoundAlias,
    "raylib::IsSoundValid" => Raylib_IsSoundValid,
    "IsSoundValid" => Raylib_IsSoundValid,
    "raylib::UpdateSound" => Raylib_UpdateSound,
    "UpdateSound" => Raylib_UpdateSound,
    "raylib::UnloadWave" => Raylib_UnloadWave,
    "UnloadWave" => Raylib_UnloadWave,
    "raylib::UnloadSound" => Raylib_UnloadSound,
    "UnloadSound" => Raylib_UnloadSound,
    "raylib::UnloadSoundAlias" => Raylib_UnloadSoundAlias,
    "UnloadSoundAlias" => Raylib_UnloadSoundAlias,
    "raylib::ExportWave" => Raylib_ExportWave,
    "ExportWave" => Raylib_ExportWave,
    "raylib::ExportWaveAsCode" => Raylib_ExportWaveAsCode,
    "ExportWaveAsCode" => Raylib_ExportWaveAsCode,
    "raylib::PlaySound" => Raylib_PlaySound,
    "PlaySound" => Raylib_PlaySound,
    "raylib::StopSound" => Raylib_StopSound,
    "StopSound" => Raylib_StopSound,
    "raylib::PauseSound" => Raylib_PauseSound,
    "PauseSound" => Raylib_PauseSound,
    "raylib::ResumeSound" => Raylib_ResumeSound,
    "ResumeSound" => Raylib_ResumeSound,
    "raylib::IsSoundPlaying" => Raylib_IsSoundPlaying,
    "IsSoundPlaying" => Raylib_IsSoundPlaying,
    "raylib::SetSoundVolume" => Raylib_SetSoundVolume,
    "SetSoundVolume" => Raylib_SetSoundVolume,
    "raylib::SetSoundPitch" => Raylib_SetSoundPitch,
    "SetSoundPitch" => Raylib_SetSoundPitch,
    "raylib::SetSoundPan" => Raylib_SetSoundPan,
    "SetSoundPan" => Raylib_SetSoundPan,
    "raylib::WaveCopy" => Raylib_WaveCopy,
    "WaveCopy" => Raylib_WaveCopy,
    "raylib::WaveCrop" => Raylib_WaveCrop,
    "WaveCrop" => Raylib_WaveCrop,
    "raylib::WaveFormat" => Raylib_WaveFormat,
    "WaveFormat" => Raylib_WaveFormat,
    "raylib::LoadWaveSamples" => Raylib_LoadWaveSamples,
    "LoadWaveSamples" => Raylib_LoadWaveSamples,
    "raylib::UnloadWaveSamples" => Raylib_UnloadWaveSamples,
    "UnloadWaveSamples" => Raylib_UnloadWaveSamples,
    "raylib::LoadMusicStream" => Raylib_LoadMusicStream,
    "LoadMusicStream" => Raylib_LoadMusicStream,
    "raylib::LoadMusicStreamFromMemory" => Raylib_LoadMusicStreamFromMemory,
    "LoadMusicStreamFromMemory" => Raylib_LoadMusicStreamFromMemory,
    "raylib::IsMusicValid" => Raylib_IsMusicValid,
    "IsMusicValid" => Raylib_IsMusicValid,
    "raylib::UnloadMusicStream" => Raylib_UnloadMusicStream,
    "UnloadMusicStream" => Raylib_UnloadMusicStream,
    "raylib::PlayMusicStream" => Raylib_PlayMusicStream,
    "PlayMusicStream" => Raylib_PlayMusicStream,
    "raylib::IsMusicStreamPlaying" => Raylib_IsMusicStreamPlaying,
    "IsMusicStreamPlaying" => Raylib_IsMusicStreamPlaying,
    "raylib::UpdateMusicStream" => Raylib_UpdateMusicStream,
    "UpdateMusicStream" => Raylib_UpdateMusicStream,
    "raylib::StopMusicStream" => Raylib_StopMusicStream,
    "StopMusicStream" => Raylib_StopMusicStream,
    "raylib::PauseMusicStream" => Raylib_PauseMusicStream,
    "PauseMusicStream" => Raylib_PauseMusicStream,
    "raylib::ResumeMusicStream" => Raylib_ResumeMusicStream,
    "ResumeMusicStream" => Raylib_ResumeMusicStream,
    "raylib::SeekMusicStream" => Raylib_SeekMusicStream,
    "SeekMusicStream" => Raylib_SeekMusicStream,
    "raylib::SetMusicVolume" => Raylib_SetMusicVolume,
    "SetMusicVolume" => Raylib_SetMusicVolume,
    "raylib::SetMusicPitch" => Raylib_SetMusicPitch,
    "SetMusicPitch" => Raylib_SetMusicPitch,
    "raylib::SetMusicPan" => Raylib_SetMusicPan,
    "SetMusicPan" => Raylib_SetMusicPan,
    "raylib::GetMusicTimeLength" => Raylib_GetMusicTimeLength,
    "GetMusicTimeLength" => Raylib_GetMusicTimeLength,
    "raylib::GetMusicTimePlayed" => Raylib_GetMusicTimePlayed,
    "GetMusicTimePlayed" => Raylib_GetMusicTimePlayed,
    "raylib::LoadAudioStream" => Raylib_LoadAudioStream,
    "LoadAudioStream" => Raylib_LoadAudioStream,
    "raylib::IsAudioStreamValid" => Raylib_IsAudioStreamValid,
    "IsAudioStreamValid" => Raylib_IsAudioStreamValid,
    "raylib::UnloadAudioStream" => Raylib_UnloadAudioStream,
    "UnloadAudioStream" => Raylib_UnloadAudioStream,
    "raylib::UpdateAudioStream" => Raylib_UpdateAudioStream,
    "UpdateAudioStream" => Raylib_UpdateAudioStream,
    "raylib::IsAudioStreamProcessed" => Raylib_IsAudioStreamProcessed,
    "IsAudioStreamProcessed" => Raylib_IsAudioStreamProcessed,
    "raylib::PlayAudioStream" => Raylib_PlayAudioStream,
    "PlayAudioStream" => Raylib_PlayAudioStream,
    "raylib::PauseAudioStream" => Raylib_PauseAudioStream,
    "PauseAudioStream" => Raylib_PauseAudioStream,
    "raylib::ResumeAudioStream" => Raylib_ResumeAudioStream,
    "ResumeAudioStream" => Raylib_ResumeAudioStream,
    "raylib::IsAudioStreamPlaying" => Raylib_IsAudioStreamPlaying,
    "IsAudioStreamPlaying" => Raylib_IsAudioStreamPlaying,
    "raylib::StopAudioStream" => Raylib_StopAudioStream,
    "StopAudioStream" => Raylib_StopAudioStream,
    "raylib::SetAudioStreamVolume" => Raylib_SetAudioStreamVolume,
    "SetAudioStreamVolume" => Raylib_SetAudioStreamVolume,
    "raylib::SetAudioStreamPitch" => Raylib_SetAudioStreamPitch,
    "SetAudioStreamPitch" => Raylib_SetAudioStreamPitch,
    "raylib::SetAudioStreamPan" => Raylib_SetAudioStreamPan,
    "SetAudioStreamPan" => Raylib_SetAudioStreamPan,
    "raylib::SetAudioStreamBufferSizeDefault" => Raylib_SetAudioStreamBufferSizeDefault,
    "SetAudioStreamBufferSizeDefault" => Raylib_SetAudioStreamBufferSizeDefault,
    "raylib::SetAudioStreamCallback" => Raylib_SetAudioStreamCallback,
    "SetAudioStreamCallback" => Raylib_SetAudioStreamCallback,
    "raylib::AttachAudioStreamProcessor" => Raylib_AttachAudioStreamProcessor,
    "AttachAudioStreamProcessor" => Raylib_AttachAudioStreamProcessor,
    "raylib::DetachAudioStreamProcessor" => Raylib_DetachAudioStreamProcessor,
    "DetachAudioStreamProcessor" => Raylib_DetachAudioStreamProcessor,
    "raylib::AttachAudioMixedProcessor" => Raylib_AttachAudioMixedProcessor,
    "AttachAudioMixedProcessor" => Raylib_AttachAudioMixedProcessor,
    "raylib::DetachAudioMixedProcessor" => Raylib_DetachAudioMixedProcessor,
    "DetachAudioMixedProcessor" => Raylib_DetachAudioMixedProcessor,
    "raylib::rlMatrixMode" => Raylib_rlMatrixMode,
    "rlMatrixMode" => Raylib_rlMatrixMode,
    "raylib::rlPushMatrix" => Raylib_rlPushMatrix,
    "rlPushMatrix" => Raylib_rlPushMatrix,
    "raylib::rlPopMatrix" => Raylib_rlPopMatrix,
    "rlPopMatrix" => Raylib_rlPopMatrix,
    "raylib::rlLoadIdentity" => Raylib_rlLoadIdentity,
    "rlLoadIdentity" => Raylib_rlLoadIdentity,
    "raylib::rlTranslatef" => Raylib_rlTranslatef,
    "rlTranslatef" => Raylib_rlTranslatef,
    "raylib::rlRotatef" => Raylib_rlRotatef,
    "rlRotatef" => Raylib_rlRotatef,
    "raylib::rlScalef" => Raylib_rlScalef,
    "rlScalef" => Raylib_rlScalef,
    "raylib::rlMultMatrixf" => Raylib_rlMultMatrixf,
    "rlMultMatrixf" => Raylib_rlMultMatrixf,
    "raylib::rlFrustum" => Raylib_rlFrustum,
    "rlFrustum" => Raylib_rlFrustum,
    "raylib::rlOrtho" => Raylib_rlOrtho,
    "rlOrtho" => Raylib_rlOrtho,
    "raylib::rlViewport" => Raylib_rlViewport,
    "rlViewport" => Raylib_rlViewport,
    "raylib::rlSetClipPlanes" => Raylib_rlSetClipPlanes,
    "rlSetClipPlanes" => Raylib_rlSetClipPlanes,
    "raylib::rlGetCullDistanceNear" => Raylib_rlGetCullDistanceNear,
    "rlGetCullDistanceNear" => Raylib_rlGetCullDistanceNear,
    "raylib::rlGetCullDistanceFar" => Raylib_rlGetCullDistanceFar,
    "rlGetCullDistanceFar" => Raylib_rlGetCullDistanceFar,
    "raylib::rlBegin" => Raylib_rlBegin,
    "rlBegin" => Raylib_rlBegin,
    "raylib::rlEnd" => Raylib_rlEnd,
    "rlEnd" => Raylib_rlEnd,
    "raylib::rlVertex2i" => Raylib_rlVertex2i,
    "rlVertex2i" => Raylib_rlVertex2i,
    "raylib::rlVertex2f" => Raylib_rlVertex2f,
    "rlVertex2f" => Raylib_rlVertex2f,
    "raylib::rlVertex3f" => Raylib_rlVertex3f,
    "rlVertex3f" => Raylib_rlVertex3f,
    "raylib::rlTexCoord2f" => Raylib_rlTexCoord2f,
    "rlTexCoord2f" => Raylib_rlTexCoord2f,
    "raylib::rlNormal3f" => Raylib_rlNormal3f,
    "rlNormal3f" => Raylib_rlNormal3f,
    "raylib::rlColor4ub" => Raylib_rlColor4ub,
    "rlColor4ub" => Raylib_rlColor4ub,
    "raylib::rlColor3f" => Raylib_rlColor3f,
    "rlColor3f" => Raylib_rlColor3f,
    "raylib::rlColor4f" => Raylib_rlColor4f,
    "rlColor4f" => Raylib_rlColor4f,
    "raylib::rlEnableVertexArray" => Raylib_rlEnableVertexArray,
    "rlEnableVertexArray" => Raylib_rlEnableVertexArray,
    "raylib::rlDisableVertexArray" => Raylib_rlDisableVertexArray,
    "rlDisableVertexArray" => Raylib_rlDisableVertexArray,
    "raylib::rlEnableVertexBuffer" => Raylib_rlEnableVertexBuffer,
    "rlEnableVertexBuffer" => Raylib_rlEnableVertexBuffer,
    "raylib::rlDisableVertexBuffer" => Raylib_rlDisableVertexBuffer,
    "rlDisableVertexBuffer" => Raylib_rlDisableVertexBuffer,
    "raylib::rlEnableVertexBufferElement" => Raylib_rlEnableVertexBufferElement,
    "rlEnableVertexBufferElement" => Raylib_rlEnableVertexBufferElement,
    "raylib::rlDisableVertexBufferElement" => Raylib_rlDisableVertexBufferElement,
    "rlDisableVertexBufferElement" => Raylib_rlDisableVertexBufferElement,
    "raylib::rlEnableVertexAttribute" => Raylib_rlEnableVertexAttribute,
    "rlEnableVertexAttribute" => Raylib_rlEnableVertexAttribute,
    "raylib::rlDisableVertexAttribute" => Raylib_rlDisableVertexAttribute,
    "rlDisableVertexAttribute" => Raylib_rlDisableVertexAttribute,
    "raylib::rlEnableStatePointer" => Raylib_rlEnableStatePointer,
    "rlEnableStatePointer" => Raylib_rlEnableStatePointer,
    "raylib::rlDisableStatePointer" => Raylib_rlDisableStatePointer,
    "rlDisableStatePointer" => Raylib_rlDisableStatePointer,
    "raylib::rlActiveTextureSlot" => Raylib_rlActiveTextureSlot,
    "rlActiveTextureSlot" => Raylib_rlActiveTextureSlot,
    "raylib::rlEnableTexture" => Raylib_rlEnableTexture,
    "rlEnableTexture" => Raylib_rlEnableTexture,
    "raylib::rlDisableTexture" => Raylib_rlDisableTexture,
    "rlDisableTexture" => Raylib_rlDisableTexture,
    "raylib::rlEnableTextureCubemap" => Raylib_rlEnableTextureCubemap,
    "rlEnableTextureCubemap" => Raylib_rlEnableTextureCubemap,
    "raylib::rlDisableTextureCubemap" => Raylib_rlDisableTextureCubemap,
    "rlDisableTextureCubemap" => Raylib_rlDisableTextureCubemap,
    "raylib::rlTextureParameters" => Raylib_rlTextureParameters,
    "rlTextureParameters" => Raylib_rlTextureParameters,
    "raylib::rlCubemapParameters" => Raylib_rlCubemapParameters,
    "rlCubemapParameters" => Raylib_rlCubemapParameters,
    "raylib::rlEnableShader" => Raylib_rlEnableShader,
    "rlEnableShader" => Raylib_rlEnableShader,
    "raylib::rlDisableShader" => Raylib_rlDisableShader,
    "rlDisableShader" => Raylib_rlDisableShader,
    "raylib::rlEnableFramebuffer" => Raylib_rlEnableFramebuffer,
    "rlEnableFramebuffer" => Raylib_rlEnableFramebuffer,
    "raylib::rlDisableFramebuffer" => Raylib_rlDisableFramebuffer,
    "rlDisableFramebuffer" => Raylib_rlDisableFramebuffer,
    "raylib::rlGetActiveFramebuffer" => Raylib_rlGetActiveFramebuffer,
    "rlGetActiveFramebuffer" => Raylib_rlGetActiveFramebuffer,
    "raylib::rlActiveDrawBuffers" => Raylib_rlActiveDrawBuffers,
    "rlActiveDrawBuffers" => Raylib_rlActiveDrawBuffers,
    "raylib::rlBlitFramebuffer" => Raylib_rlBlitFramebuffer,
    "rlBlitFramebuffer" => Raylib_rlBlitFramebuffer,
    "raylib::rlBindFramebuffer" => Raylib_rlBindFramebuffer,
    "rlBindFramebuffer" => Raylib_rlBindFramebuffer,
    "raylib::rlEnableColorBlend" => Raylib_rlEnableColorBlend,
    "rlEnableColorBlend" => Raylib_rlEnableColorBlend,
    "raylib::rlDisableColorBlend" => Raylib_rlDisableColorBlend,
    "rlDisableColorBlend" => Raylib_rlDisableColorBlend,
    "raylib::rlEnableDepthTest" => Raylib_rlEnableDepthTest,
    "rlEnableDepthTest" => Raylib_rlEnableDepthTest,
    "raylib::rlDisableDepthTest" => Raylib_rlDisableDepthTest,
    "rlDisableDepthTest" => Raylib_rlDisableDepthTest,
    "raylib::rlEnableDepthMask" => Raylib_rlEnableDepthMask,
    "rlEnableDepthMask" => Raylib_rlEnableDepthMask,
    "raylib::rlDisableDepthMask" => Raylib_rlDisableDepthMask,
    "rlDisableDepthMask" => Raylib_rlDisableDepthMask,
    "raylib::rlEnableBackfaceCulling" => Raylib_rlEnableBackfaceCulling,
    "rlEnableBackfaceCulling" => Raylib_rlEnableBackfaceCulling,
    "raylib::rlDisableBackfaceCulling" => Raylib_rlDisableBackfaceCulling,
    "rlDisableBackfaceCulling" => Raylib_rlDisableBackfaceCulling,
    "raylib::rlColorMask" => Raylib_rlColorMask,
    "rlColorMask" => Raylib_rlColorMask,
    "raylib::rlSetCullFace" => Raylib_rlSetCullFace,
    "rlSetCullFace" => Raylib_rlSetCullFace,
    "raylib::rlEnableScissorTest" => Raylib_rlEnableScissorTest,
    "rlEnableScissorTest" => Raylib_rlEnableScissorTest,
    "raylib::rlDisableScissorTest" => Raylib_rlDisableScissorTest,
    "rlDisableScissorTest" => Raylib_rlDisableScissorTest,
    "raylib::rlScissor" => Raylib_rlScissor,
    "rlScissor" => Raylib_rlScissor,
    "raylib::rlEnablePointMode" => Raylib_rlEnablePointMode,
    "rlEnablePointMode" => Raylib_rlEnablePointMode,
    "raylib::rlDisablePointMode" => Raylib_rlDisablePointMode,
    "rlDisablePointMode" => Raylib_rlDisablePointMode,
    "raylib::rlSetPointSize" => Raylib_rlSetPointSize,
    "rlSetPointSize" => Raylib_rlSetPointSize,
    "raylib::rlGetPointSize" => Raylib_rlGetPointSize,
    "rlGetPointSize" => Raylib_rlGetPointSize,
    "raylib::rlEnableWireMode" => Raylib_rlEnableWireMode,
    "rlEnableWireMode" => Raylib_rlEnableWireMode,
    "raylib::rlDisableWireMode" => Raylib_rlDisableWireMode,
    "rlDisableWireMode" => Raylib_rlDisableWireMode,
    "raylib::rlSetLineWidth" => Raylib_rlSetLineWidth,
    "rlSetLineWidth" => Raylib_rlSetLineWidth,
    "raylib::rlGetLineWidth" => Raylib_rlGetLineWidth,
    "rlGetLineWidth" => Raylib_rlGetLineWidth,
    "raylib::rlEnableSmoothLines" => Raylib_rlEnableSmoothLines,
    "rlEnableSmoothLines" => Raylib_rlEnableSmoothLines,
    "raylib::rlDisableSmoothLines" => Raylib_rlDisableSmoothLines,
    "rlDisableSmoothLines" => Raylib_rlDisableSmoothLines,
    "raylib::rlEnableStereoRender" => Raylib_rlEnableStereoRender,
    "rlEnableStereoRender" => Raylib_rlEnableStereoRender,
    "raylib::rlDisableStereoRender" => Raylib_rlDisableStereoRender,
    "rlDisableStereoRender" => Raylib_rlDisableStereoRender,
    "raylib::rlIsStereoRenderEnabled" => Raylib_rlIsStereoRenderEnabled,
    "rlIsStereoRenderEnabled" => Raylib_rlIsStereoRenderEnabled,
    "raylib::rlClearColor" => Raylib_rlClearColor,
    "rlClearColor" => Raylib_rlClearColor,
    "raylib::rlClearScreenBuffers" => Raylib_rlClearScreenBuffers,
    "rlClearScreenBuffers" => Raylib_rlClearScreenBuffers,
    "raylib::rlCheckErrors" => Raylib_rlCheckErrors,
    "rlCheckErrors" => Raylib_rlCheckErrors,
    "raylib::rlSetBlendMode" => Raylib_rlSetBlendMode,
    "rlSetBlendMode" => Raylib_rlSetBlendMode,
    "raylib::rlSetBlendFactors" => Raylib_rlSetBlendFactors,
    "rlSetBlendFactors" => Raylib_rlSetBlendFactors,
    "raylib::rlSetBlendFactorsSeparate" => Raylib_rlSetBlendFactorsSeparate,
    "rlSetBlendFactorsSeparate" => Raylib_rlSetBlendFactorsSeparate,
    "raylib::rlglInit" => Raylib_rlglInit,
    "rlglInit" => Raylib_rlglInit,
    "raylib::rlglClose" => Raylib_rlglClose,
    "rlglClose" => Raylib_rlglClose,
    "raylib::rlLoadExtensions" => Raylib_rlLoadExtensions,
    "rlLoadExtensions" => Raylib_rlLoadExtensions,
    "raylib::rlGetProcAddress" => Raylib_rlGetProcAddress,
    "rlGetProcAddress" => Raylib_rlGetProcAddress,
    "raylib::rlGetVersion" => Raylib_rlGetVersion,
    "rlGetVersion" => Raylib_rlGetVersion,
    "raylib::rlSetFramebufferWidth" => Raylib_rlSetFramebufferWidth,
    "rlSetFramebufferWidth" => Raylib_rlSetFramebufferWidth,
    "raylib::rlGetFramebufferWidth" => Raylib_rlGetFramebufferWidth,
    "rlGetFramebufferWidth" => Raylib_rlGetFramebufferWidth,
    "raylib::rlSetFramebufferHeight" => Raylib_rlSetFramebufferHeight,
    "rlSetFramebufferHeight" => Raylib_rlSetFramebufferHeight,
    "raylib::rlGetFramebufferHeight" => Raylib_rlGetFramebufferHeight,
    "rlGetFramebufferHeight" => Raylib_rlGetFramebufferHeight,
    "raylib::rlGetTextureIdDefault" => Raylib_rlGetTextureIdDefault,
    "rlGetTextureIdDefault" => Raylib_rlGetTextureIdDefault,
    "raylib::rlGetShaderIdDefault" => Raylib_rlGetShaderIdDefault,
    "rlGetShaderIdDefault" => Raylib_rlGetShaderIdDefault,
    "raylib::rlGetShaderLocsDefault" => Raylib_rlGetShaderLocsDefault,
    "rlGetShaderLocsDefault" => Raylib_rlGetShaderLocsDefault,
    "raylib::rlLoadRenderBatch" => Raylib_rlLoadRenderBatch,
    "rlLoadRenderBatch" => Raylib_rlLoadRenderBatch,
    "raylib::rlUnloadRenderBatch" => Raylib_rlUnloadRenderBatch,
    "rlUnloadRenderBatch" => Raylib_rlUnloadRenderBatch,
    "raylib::rlDrawRenderBatch" => Raylib_rlDrawRenderBatch,
    "rlDrawRenderBatch" => Raylib_rlDrawRenderBatch,
    "raylib::rlSetRenderBatchActive" => Raylib_rlSetRenderBatchActive,
    "rlSetRenderBatchActive" => Raylib_rlSetRenderBatchActive,
    "raylib::rlDrawRenderBatchActive" => Raylib_rlDrawRenderBatchActive,
    "rlDrawRenderBatchActive" => Raylib_rlDrawRenderBatchActive,
    "raylib::rlCheckRenderBatchLimit" => Raylib_rlCheckRenderBatchLimit,
    "rlCheckRenderBatchLimit" => Raylib_rlCheckRenderBatchLimit,
    "raylib::rlSetTexture" => Raylib_rlSetTexture,
    "rlSetTexture" => Raylib_rlSetTexture,
    "raylib::rlLoadVertexArray" => Raylib_rlLoadVertexArray,
    "rlLoadVertexArray" => Raylib_rlLoadVertexArray,
    "raylib::rlLoadVertexBuffer" => Raylib_rlLoadVertexBuffer,
    "rlLoadVertexBuffer" => Raylib_rlLoadVertexBuffer,
    "raylib::rlLoadVertexBufferElement" => Raylib_rlLoadVertexBufferElement,
    "rlLoadVertexBufferElement" => Raylib_rlLoadVertexBufferElement,
    "raylib::rlUpdateVertexBuffer" => Raylib_rlUpdateVertexBuffer,
    "rlUpdateVertexBuffer" => Raylib_rlUpdateVertexBuffer,
    "raylib::rlUpdateVertexBufferElements" => Raylib_rlUpdateVertexBufferElements,
    "rlUpdateVertexBufferElements" => Raylib_rlUpdateVertexBufferElements,
    "raylib::rlUnloadVertexArray" => Raylib_rlUnloadVertexArray,
    "rlUnloadVertexArray" => Raylib_rlUnloadVertexArray,
    "raylib::rlUnloadVertexBuffer" => Raylib_rlUnloadVertexBuffer,
    "rlUnloadVertexBuffer" => Raylib_rlUnloadVertexBuffer,
    "raylib::rlSetVertexAttribute" => Raylib_rlSetVertexAttribute,
    "rlSetVertexAttribute" => Raylib_rlSetVertexAttribute,
    "raylib::rlSetVertexAttributeDivisor" => Raylib_rlSetVertexAttributeDivisor,
    "rlSetVertexAttributeDivisor" => Raylib_rlSetVertexAttributeDivisor,
    "raylib::rlSetVertexAttributeDefault" => Raylib_rlSetVertexAttributeDefault,
    "rlSetVertexAttributeDefault" => Raylib_rlSetVertexAttributeDefault,
    "raylib::rlDrawVertexArray" => Raylib_rlDrawVertexArray,
    "rlDrawVertexArray" => Raylib_rlDrawVertexArray,
    "raylib::rlDrawVertexArrayElements" => Raylib_rlDrawVertexArrayElements,
    "rlDrawVertexArrayElements" => Raylib_rlDrawVertexArrayElements,
    "raylib::rlDrawVertexArrayInstanced" => Raylib_rlDrawVertexArrayInstanced,
    "rlDrawVertexArrayInstanced" => Raylib_rlDrawVertexArrayInstanced,
    "raylib::rlDrawVertexArrayElementsInstanced" => Raylib_rlDrawVertexArrayElementsInstanced,
    "rlDrawVertexArrayElementsInstanced" => Raylib_rlDrawVertexArrayElementsInstanced,
    "raylib::rlLoadTexture" => Raylib_rlLoadTexture,
    "rlLoadTexture" => Raylib_rlLoadTexture,
    "raylib::rlLoadTextureDepth" => Raylib_rlLoadTextureDepth,
    "rlLoadTextureDepth" => Raylib_rlLoadTextureDepth,
    "raylib::rlLoadTextureCubemap" => Raylib_rlLoadTextureCubemap,
    "rlLoadTextureCubemap" => Raylib_rlLoadTextureCubemap,
    "raylib::rlUpdateTexture" => Raylib_rlUpdateTexture,
    "rlUpdateTexture" => Raylib_rlUpdateTexture,
    "raylib::rlGetGlTextureFormats" => Raylib_rlGetGlTextureFormats,
    "rlGetGlTextureFormats" => Raylib_rlGetGlTextureFormats,
    "raylib::rlGetPixelFormatName" => Raylib_rlGetPixelFormatName,
    "rlGetPixelFormatName" => Raylib_rlGetPixelFormatName,
    "raylib::rlUnloadTexture" => Raylib_rlUnloadTexture,
    "rlUnloadTexture" => Raylib_rlUnloadTexture,
    "raylib::rlGenTextureMipmaps" => Raylib_rlGenTextureMipmaps,
    "rlGenTextureMipmaps" => Raylib_rlGenTextureMipmaps,
    "raylib::rlReadTexturePixels" => Raylib_rlReadTexturePixels,
    "rlReadTexturePixels" => Raylib_rlReadTexturePixels,
    "raylib::rlReadScreenPixels" => Raylib_rlReadScreenPixels,
    "rlReadScreenPixels" => Raylib_rlReadScreenPixels,
    "raylib::rlLoadFramebuffer" => Raylib_rlLoadFramebuffer,
    "rlLoadFramebuffer" => Raylib_rlLoadFramebuffer,
    "raylib::rlFramebufferAttach" => Raylib_rlFramebufferAttach,
    "rlFramebufferAttach" => Raylib_rlFramebufferAttach,
    "raylib::rlFramebufferComplete" => Raylib_rlFramebufferComplete,
    "rlFramebufferComplete" => Raylib_rlFramebufferComplete,
    "raylib::rlUnloadFramebuffer" => Raylib_rlUnloadFramebuffer,
    "rlUnloadFramebuffer" => Raylib_rlUnloadFramebuffer,
    "raylib::rlCopyFramebuffer" => Raylib_rlCopyFramebuffer,
    "rlCopyFramebuffer" => Raylib_rlCopyFramebuffer,
    "raylib::rlResizeFramebuffer" => Raylib_rlResizeFramebuffer,
    "rlResizeFramebuffer" => Raylib_rlResizeFramebuffer,
    "raylib::rlLoadShader" => Raylib_rlLoadShader,
    "rlLoadShader" => Raylib_rlLoadShader,
    "raylib::rlLoadShaderProgram" => Raylib_rlLoadShaderProgram,
    "rlLoadShaderProgram" => Raylib_rlLoadShaderProgram,
    "raylib::rlLoadShaderProgramEx" => Raylib_rlLoadShaderProgramEx,
    "rlLoadShaderProgramEx" => Raylib_rlLoadShaderProgramEx,
    "raylib::rlLoadShaderProgramCompute" => Raylib_rlLoadShaderProgramCompute,
    "rlLoadShaderProgramCompute" => Raylib_rlLoadShaderProgramCompute,
    "raylib::rlUnloadShader" => Raylib_rlUnloadShader,
    "rlUnloadShader" => Raylib_rlUnloadShader,
    "raylib::rlUnloadShaderProgram" => Raylib_rlUnloadShaderProgram,
    "rlUnloadShaderProgram" => Raylib_rlUnloadShaderProgram,
    "raylib::rlGetLocationUniform" => Raylib_rlGetLocationUniform,
    "rlGetLocationUniform" => Raylib_rlGetLocationUniform,
    "raylib::rlGetLocationAttrib" => Raylib_rlGetLocationAttrib,
    "rlGetLocationAttrib" => Raylib_rlGetLocationAttrib,
    "raylib::rlSetUniform" => Raylib_rlSetUniform,
    "rlSetUniform" => Raylib_rlSetUniform,
    "raylib::rlSetUniformMatrix" => Raylib_rlSetUniformMatrix,
    "rlSetUniformMatrix" => Raylib_rlSetUniformMatrix,
    "raylib::rlSetUniformMatrices" => Raylib_rlSetUniformMatrices,
    "rlSetUniformMatrices" => Raylib_rlSetUniformMatrices,
    "raylib::rlSetUniformSampler" => Raylib_rlSetUniformSampler,
    "rlSetUniformSampler" => Raylib_rlSetUniformSampler,
    "raylib::rlSetShader" => Raylib_rlSetShader,
    "rlSetShader" => Raylib_rlSetShader,
    "raylib::rlComputeShaderDispatch" => Raylib_rlComputeShaderDispatch,
    "rlComputeShaderDispatch" => Raylib_rlComputeShaderDispatch,
    "raylib::rlLoadShaderBuffer" => Raylib_rlLoadShaderBuffer,
    "rlLoadShaderBuffer" => Raylib_rlLoadShaderBuffer,
    "raylib::rlUnloadShaderBuffer" => Raylib_rlUnloadShaderBuffer,
    "rlUnloadShaderBuffer" => Raylib_rlUnloadShaderBuffer,
    "raylib::rlUpdateShaderBuffer" => Raylib_rlUpdateShaderBuffer,
    "rlUpdateShaderBuffer" => Raylib_rlUpdateShaderBuffer,
    "raylib::rlBindShaderBuffer" => Raylib_rlBindShaderBuffer,
    "rlBindShaderBuffer" => Raylib_rlBindShaderBuffer,
    "raylib::rlReadShaderBuffer" => Raylib_rlReadShaderBuffer,
    "rlReadShaderBuffer" => Raylib_rlReadShaderBuffer,
    "raylib::rlCopyShaderBuffer" => Raylib_rlCopyShaderBuffer,
    "rlCopyShaderBuffer" => Raylib_rlCopyShaderBuffer,
    "raylib::rlGetShaderBufferSize" => Raylib_rlGetShaderBufferSize,
    "rlGetShaderBufferSize" => Raylib_rlGetShaderBufferSize,
    "raylib::rlBindImageTexture" => Raylib_rlBindImageTexture,
    "rlBindImageTexture" => Raylib_rlBindImageTexture,
    "raylib::rlGetMatrixModelview" => Raylib_rlGetMatrixModelview,
    "rlGetMatrixModelview" => Raylib_rlGetMatrixModelview,
    "raylib::rlGetMatrixProjection" => Raylib_rlGetMatrixProjection,
    "rlGetMatrixProjection" => Raylib_rlGetMatrixProjection,
    "raylib::rlGetMatrixTransform" => Raylib_rlGetMatrixTransform,
    "rlGetMatrixTransform" => Raylib_rlGetMatrixTransform,
    "raylib::rlGetMatrixProjectionStereo" => Raylib_rlGetMatrixProjectionStereo,
    "rlGetMatrixProjectionStereo" => Raylib_rlGetMatrixProjectionStereo,
    "raylib::rlGetMatrixViewOffsetStereo" => Raylib_rlGetMatrixViewOffsetStereo,
    "rlGetMatrixViewOffsetStereo" => Raylib_rlGetMatrixViewOffsetStereo,
    "raylib::rlSetMatrixProjection" => Raylib_rlSetMatrixProjection,
    "rlSetMatrixProjection" => Raylib_rlSetMatrixProjection,
    "raylib::rlSetMatrixModelview" => Raylib_rlSetMatrixModelview,
    "rlSetMatrixModelview" => Raylib_rlSetMatrixModelview,
    "raylib::rlSetMatrixProjectionStereo" => Raylib_rlSetMatrixProjectionStereo,
    "rlSetMatrixProjectionStereo" => Raylib_rlSetMatrixProjectionStereo,
    "raylib::rlSetMatrixViewOffsetStereo" => Raylib_rlSetMatrixViewOffsetStereo,
    "rlSetMatrixViewOffsetStereo" => Raylib_rlSetMatrixViewOffsetStereo,
    "raylib::rlLoadDrawCube" => Raylib_rlLoadDrawCube,
    "rlLoadDrawCube" => Raylib_rlLoadDrawCube,
    "raylib::rlLoadDrawQuad" => Raylib_rlLoadDrawQuad,
    "rlLoadDrawQuad" => Raylib_rlLoadDrawQuad,
    "raylib::Clamp" => Raylib_Clamp,
    "Clamp" => Raylib_Clamp,
    "raylib::Lerp" => Raylib_Lerp,
    "Lerp" => Raylib_Lerp,
    "raylib::Normalize" => Raylib_Normalize,
    "Normalize" => Raylib_Normalize,
    "raylib::Remap" => Raylib_Remap,
    "Remap" => Raylib_Remap,
    "raylib::Wrap" => Raylib_Wrap,
    "Wrap" => Raylib_Wrap,
    "raylib::FloatEquals" => Raylib_FloatEquals,
    "FloatEquals" => Raylib_FloatEquals,
    "raylib::Vector2Zero" => Raylib_Vector2Zero,
    "Vector2Zero" => Raylib_Vector2Zero,
    "raylib::Vector2One" => Raylib_Vector2One,
    "Vector2One" => Raylib_Vector2One,
    "raylib::Vector2Add" => Raylib_Vector2Add,
    "Vector2Add" => Raylib_Vector2Add,
    "raylib::Vector2AddValue" => Raylib_Vector2AddValue,
    "Vector2AddValue" => Raylib_Vector2AddValue,
    "raylib::Vector2Subtract" => Raylib_Vector2Subtract,
    "Vector2Subtract" => Raylib_Vector2Subtract,
    "raylib::Vector2SubtractValue" => Raylib_Vector2SubtractValue,
    "Vector2SubtractValue" => Raylib_Vector2SubtractValue,
    "raylib::Vector2Length" => Raylib_Vector2Length,
    "Vector2Length" => Raylib_Vector2Length,
    "raylib::Vector2LengthSqr" => Raylib_Vector2LengthSqr,
    "Vector2LengthSqr" => Raylib_Vector2LengthSqr,
    "raylib::Vector2DotProduct" => Raylib_Vector2DotProduct,
    "Vector2DotProduct" => Raylib_Vector2DotProduct,
    "raylib::Vector2CrossProduct" => Raylib_Vector2CrossProduct,
    "Vector2CrossProduct" => Raylib_Vector2CrossProduct,
    "raylib::Vector2Distance" => Raylib_Vector2Distance,
    "Vector2Distance" => Raylib_Vector2Distance,
    "raylib::Vector2DistanceSqr" => Raylib_Vector2DistanceSqr,
    "Vector2DistanceSqr" => Raylib_Vector2DistanceSqr,
    "raylib::Vector2Angle" => Raylib_Vector2Angle,
    "Vector2Angle" => Raylib_Vector2Angle,
    "raylib::Vector2LineAngle" => Raylib_Vector2LineAngle,
    "Vector2LineAngle" => Raylib_Vector2LineAngle,
    "raylib::Vector2Scale" => Raylib_Vector2Scale,
    "Vector2Scale" => Raylib_Vector2Scale,
    "raylib::Vector2Multiply" => Raylib_Vector2Multiply,
    "Vector2Multiply" => Raylib_Vector2Multiply,
    "raylib::Vector2Negate" => Raylib_Vector2Negate,
    "Vector2Negate" => Raylib_Vector2Negate,
    "raylib::Vector2Divide" => Raylib_Vector2Divide,
    "Vector2Divide" => Raylib_Vector2Divide,
    "raylib::Vector2Normalize" => Raylib_Vector2Normalize,
    "Vector2Normalize" => Raylib_Vector2Normalize,
    "raylib::Vector2Transform" => Raylib_Vector2Transform,
    "Vector2Transform" => Raylib_Vector2Transform,
    "raylib::Vector2Lerp" => Raylib_Vector2Lerp,
    "Vector2Lerp" => Raylib_Vector2Lerp,
    "raylib::Vector2Reflect" => Raylib_Vector2Reflect,
    "Vector2Reflect" => Raylib_Vector2Reflect,
    "raylib::Vector2Min" => Raylib_Vector2Min,
    "Vector2Min" => Raylib_Vector2Min,
    "raylib::Vector2Max" => Raylib_Vector2Max,
    "Vector2Max" => Raylib_Vector2Max,
    "raylib::Vector2Rotate" => Raylib_Vector2Rotate,
    "Vector2Rotate" => Raylib_Vector2Rotate,
    "raylib::Vector2MoveTowards" => Raylib_Vector2MoveTowards,
    "Vector2MoveTowards" => Raylib_Vector2MoveTowards,
    "raylib::Vector2Invert" => Raylib_Vector2Invert,
    "Vector2Invert" => Raylib_Vector2Invert,
    "raylib::Vector2Clamp" => Raylib_Vector2Clamp,
    "Vector2Clamp" => Raylib_Vector2Clamp,
    "raylib::Vector2ClampValue" => Raylib_Vector2ClampValue,
    "Vector2ClampValue" => Raylib_Vector2ClampValue,
    "raylib::Vector2Equals" => Raylib_Vector2Equals,
    "Vector2Equals" => Raylib_Vector2Equals,
    "raylib::Vector2Refract" => Raylib_Vector2Refract,
    "Vector2Refract" => Raylib_Vector2Refract,
    "raylib::Vector3Zero" => Raylib_Vector3Zero,
    "Vector3Zero" => Raylib_Vector3Zero,
    "raylib::Vector3One" => Raylib_Vector3One,
    "Vector3One" => Raylib_Vector3One,
    "raylib::Vector3Add" => Raylib_Vector3Add,
    "Vector3Add" => Raylib_Vector3Add,
    "raylib::Vector3AddValue" => Raylib_Vector3AddValue,
    "Vector3AddValue" => Raylib_Vector3AddValue,
    "raylib::Vector3Subtract" => Raylib_Vector3Subtract,
    "Vector3Subtract" => Raylib_Vector3Subtract,
    "raylib::Vector3SubtractValue" => Raylib_Vector3SubtractValue,
    "Vector3SubtractValue" => Raylib_Vector3SubtractValue,
    "raylib::Vector3Scale" => Raylib_Vector3Scale,
    "Vector3Scale" => Raylib_Vector3Scale,
    "raylib::Vector3Multiply" => Raylib_Vector3Multiply,
    "Vector3Multiply" => Raylib_Vector3Multiply,
    "raylib::Vector3CrossProduct" => Raylib_Vector3CrossProduct,
    "Vector3CrossProduct" => Raylib_Vector3CrossProduct,
    "raylib::Vector3Perpendicular" => Raylib_Vector3Perpendicular,
    "Vector3Perpendicular" => Raylib_Vector3Perpendicular,
    "raylib::Vector3Length" => Raylib_Vector3Length,
    "Vector3Length" => Raylib_Vector3Length,
    "raylib::Vector3LengthSqr" => Raylib_Vector3LengthSqr,
    "Vector3LengthSqr" => Raylib_Vector3LengthSqr,
    "raylib::Vector3DotProduct" => Raylib_Vector3DotProduct,
    "Vector3DotProduct" => Raylib_Vector3DotProduct,
    "raylib::Vector3Distance" => Raylib_Vector3Distance,
    "Vector3Distance" => Raylib_Vector3Distance,
    "raylib::Vector3DistanceSqr" => Raylib_Vector3DistanceSqr,
    "Vector3DistanceSqr" => Raylib_Vector3DistanceSqr,
    "raylib::Vector3Angle" => Raylib_Vector3Angle,
    "Vector3Angle" => Raylib_Vector3Angle,
    "raylib::Vector3Negate" => Raylib_Vector3Negate,
    "Vector3Negate" => Raylib_Vector3Negate,
    "raylib::Vector3Divide" => Raylib_Vector3Divide,
    "Vector3Divide" => Raylib_Vector3Divide,
    "raylib::Vector3Normalize" => Raylib_Vector3Normalize,
    "Vector3Normalize" => Raylib_Vector3Normalize,
    "raylib::Vector3Project" => Raylib_Vector3Project,
    "Vector3Project" => Raylib_Vector3Project,
    "raylib::Vector3Reject" => Raylib_Vector3Reject,
    "Vector3Reject" => Raylib_Vector3Reject,
    "raylib::Vector3OrthoNormalize" => Raylib_Vector3OrthoNormalize,
    "Vector3OrthoNormalize" => Raylib_Vector3OrthoNormalize,
    "raylib::Vector3Transform" => Raylib_Vector3Transform,
    "Vector3Transform" => Raylib_Vector3Transform,
    "raylib::Vector3RotateByQuaternion" => Raylib_Vector3RotateByQuaternion,
    "Vector3RotateByQuaternion" => Raylib_Vector3RotateByQuaternion,
    "raylib::Vector3RotateByAxisAngle" => Raylib_Vector3RotateByAxisAngle,
    "Vector3RotateByAxisAngle" => Raylib_Vector3RotateByAxisAngle,
    "raylib::Vector3MoveTowards" => Raylib_Vector3MoveTowards,
    "Vector3MoveTowards" => Raylib_Vector3MoveTowards,
    "raylib::Vector3Lerp" => Raylib_Vector3Lerp,
    "Vector3Lerp" => Raylib_Vector3Lerp,
    "raylib::Vector3CubicHermite" => Raylib_Vector3CubicHermite,
    "Vector3CubicHermite" => Raylib_Vector3CubicHermite,
    "raylib::Vector3Reflect" => Raylib_Vector3Reflect,
    "Vector3Reflect" => Raylib_Vector3Reflect,
    "raylib::Vector3Min" => Raylib_Vector3Min,
    "Vector3Min" => Raylib_Vector3Min,
    "raylib::Vector3Max" => Raylib_Vector3Max,
    "Vector3Max" => Raylib_Vector3Max,
    "raylib::Vector3Barycenter" => Raylib_Vector3Barycenter,
    "Vector3Barycenter" => Raylib_Vector3Barycenter,
    "raylib::Vector3Unproject" => Raylib_Vector3Unproject,
    "Vector3Unproject" => Raylib_Vector3Unproject,
    "raylib::Vector3ToFloatV" => Raylib_Vector3ToFloatV,
    "Vector3ToFloatV" => Raylib_Vector3ToFloatV,
    "raylib::Vector3Invert" => Raylib_Vector3Invert,
    "Vector3Invert" => Raylib_Vector3Invert,
    "raylib::Vector3Clamp" => Raylib_Vector3Clamp,
    "Vector3Clamp" => Raylib_Vector3Clamp,
    "raylib::Vector3ClampValue" => Raylib_Vector3ClampValue,
    "Vector3ClampValue" => Raylib_Vector3ClampValue,
    "raylib::Vector3Equals" => Raylib_Vector3Equals,
    "Vector3Equals" => Raylib_Vector3Equals,
    "raylib::Vector3Refract" => Raylib_Vector3Refract,
    "Vector3Refract" => Raylib_Vector3Refract,
    "raylib::Vector4Zero" => Raylib_Vector4Zero,
    "Vector4Zero" => Raylib_Vector4Zero,
    "raylib::Vector4One" => Raylib_Vector4One,
    "Vector4One" => Raylib_Vector4One,
    "raylib::Vector4Add" => Raylib_Vector4Add,
    "Vector4Add" => Raylib_Vector4Add,
    "raylib::Vector4AddValue" => Raylib_Vector4AddValue,
    "Vector4AddValue" => Raylib_Vector4AddValue,
    "raylib::Vector4Subtract" => Raylib_Vector4Subtract,
    "Vector4Subtract" => Raylib_Vector4Subtract,
    "raylib::Vector4SubtractValue" => Raylib_Vector4SubtractValue,
    "Vector4SubtractValue" => Raylib_Vector4SubtractValue,
    "raylib::Vector4Length" => Raylib_Vector4Length,
    "Vector4Length" => Raylib_Vector4Length,
    "raylib::Vector4LengthSqr" => Raylib_Vector4LengthSqr,
    "Vector4LengthSqr" => Raylib_Vector4LengthSqr,
    "raylib::Vector4DotProduct" => Raylib_Vector4DotProduct,
    "Vector4DotProduct" => Raylib_Vector4DotProduct,
    "raylib::Vector4Distance" => Raylib_Vector4Distance,
    "Vector4Distance" => Raylib_Vector4Distance,
    "raylib::Vector4DistanceSqr" => Raylib_Vector4DistanceSqr,
    "Vector4DistanceSqr" => Raylib_Vector4DistanceSqr,
    "raylib::Vector4Scale" => Raylib_Vector4Scale,
    "Vector4Scale" => Raylib_Vector4Scale,
    "raylib::Vector4Multiply" => Raylib_Vector4Multiply,
    "Vector4Multiply" => Raylib_Vector4Multiply,
    "raylib::Vector4Negate" => Raylib_Vector4Negate,
    "Vector4Negate" => Raylib_Vector4Negate,
    "raylib::Vector4Divide" => Raylib_Vector4Divide,
    "Vector4Divide" => Raylib_Vector4Divide,
    "raylib::Vector4Normalize" => Raylib_Vector4Normalize,
    "Vector4Normalize" => Raylib_Vector4Normalize,
    "raylib::Vector4Min" => Raylib_Vector4Min,
    "Vector4Min" => Raylib_Vector4Min,
    "raylib::Vector4Max" => Raylib_Vector4Max,
    "Vector4Max" => Raylib_Vector4Max,
    "raylib::Vector4Lerp" => Raylib_Vector4Lerp,
    "Vector4Lerp" => Raylib_Vector4Lerp,
    "raylib::Vector4MoveTowards" => Raylib_Vector4MoveTowards,
    "Vector4MoveTowards" => Raylib_Vector4MoveTowards,
    "raylib::Vector4Invert" => Raylib_Vector4Invert,
    "Vector4Invert" => Raylib_Vector4Invert,
    "raylib::Vector4Equals" => Raylib_Vector4Equals,
    "Vector4Equals" => Raylib_Vector4Equals,
    "raylib::MatrixDeterminant" => Raylib_MatrixDeterminant,
    "MatrixDeterminant" => Raylib_MatrixDeterminant,
    "raylib::MatrixTrace" => Raylib_MatrixTrace,
    "MatrixTrace" => Raylib_MatrixTrace,
    "raylib::MatrixTranspose" => Raylib_MatrixTranspose,
    "MatrixTranspose" => Raylib_MatrixTranspose,
    "raylib::MatrixInvert" => Raylib_MatrixInvert,
    "MatrixInvert" => Raylib_MatrixInvert,
    "raylib::MatrixIdentity" => Raylib_MatrixIdentity,
    "MatrixIdentity" => Raylib_MatrixIdentity,
    "raylib::MatrixAdd" => Raylib_MatrixAdd,
    "MatrixAdd" => Raylib_MatrixAdd,
    "raylib::MatrixSubtract" => Raylib_MatrixSubtract,
    "MatrixSubtract" => Raylib_MatrixSubtract,
    "raylib::MatrixMultiply" => Raylib_MatrixMultiply,
    "MatrixMultiply" => Raylib_MatrixMultiply,
    "raylib::MatrixMultiplyValue" => Raylib_MatrixMultiplyValue,
    "MatrixMultiplyValue" => Raylib_MatrixMultiplyValue,
    "raylib::MatrixTranslate" => Raylib_MatrixTranslate,
    "MatrixTranslate" => Raylib_MatrixTranslate,
    "raylib::MatrixRotate" => Raylib_MatrixRotate,
    "MatrixRotate" => Raylib_MatrixRotate,
    "raylib::MatrixRotateX" => Raylib_MatrixRotateX,
    "MatrixRotateX" => Raylib_MatrixRotateX,
    "raylib::MatrixRotateY" => Raylib_MatrixRotateY,
    "MatrixRotateY" => Raylib_MatrixRotateY,
    "raylib::MatrixRotateZ" => Raylib_MatrixRotateZ,
    "MatrixRotateZ" => Raylib_MatrixRotateZ,
    "raylib::MatrixRotateXYZ" => Raylib_MatrixRotateXYZ,
    "MatrixRotateXYZ" => Raylib_MatrixRotateXYZ,
    "raylib::MatrixRotateZYX" => Raylib_MatrixRotateZYX,
    "MatrixRotateZYX" => Raylib_MatrixRotateZYX,
    "raylib::MatrixScale" => Raylib_MatrixScale,
    "MatrixScale" => Raylib_MatrixScale,
    "raylib::MatrixFrustum" => Raylib_MatrixFrustum,
    "MatrixFrustum" => Raylib_MatrixFrustum,
    "raylib::MatrixPerspective" => Raylib_MatrixPerspective,
    "MatrixPerspective" => Raylib_MatrixPerspective,
    "raylib::MatrixOrtho" => Raylib_MatrixOrtho,
    "MatrixOrtho" => Raylib_MatrixOrtho,
    "raylib::MatrixLookAt" => Raylib_MatrixLookAt,
    "MatrixLookAt" => Raylib_MatrixLookAt,
    "raylib::MatrixToFloatV" => Raylib_MatrixToFloatV,
    "MatrixToFloatV" => Raylib_MatrixToFloatV,
    "raylib::QuaternionAdd" => Raylib_QuaternionAdd,
    "QuaternionAdd" => Raylib_QuaternionAdd,
    "raylib::QuaternionAddValue" => Raylib_QuaternionAddValue,
    "QuaternionAddValue" => Raylib_QuaternionAddValue,
    "raylib::QuaternionSubtract" => Raylib_QuaternionSubtract,
    "QuaternionSubtract" => Raylib_QuaternionSubtract,
    "raylib::QuaternionSubtractValue" => Raylib_QuaternionSubtractValue,
    "QuaternionSubtractValue" => Raylib_QuaternionSubtractValue,
    "raylib::QuaternionIdentity" => Raylib_QuaternionIdentity,
    "QuaternionIdentity" => Raylib_QuaternionIdentity,
    "raylib::QuaternionLength" => Raylib_QuaternionLength,
    "QuaternionLength" => Raylib_QuaternionLength,
    "raylib::QuaternionNormalize" => Raylib_QuaternionNormalize,
    "QuaternionNormalize" => Raylib_QuaternionNormalize,
    "raylib::QuaternionInvert" => Raylib_QuaternionInvert,
    "QuaternionInvert" => Raylib_QuaternionInvert,
    "raylib::QuaternionMultiply" => Raylib_QuaternionMultiply,
    "QuaternionMultiply" => Raylib_QuaternionMultiply,
    "raylib::QuaternionScale" => Raylib_QuaternionScale,
    "QuaternionScale" => Raylib_QuaternionScale,
    "raylib::QuaternionDivide" => Raylib_QuaternionDivide,
    "QuaternionDivide" => Raylib_QuaternionDivide,
    "raylib::QuaternionLerp" => Raylib_QuaternionLerp,
    "QuaternionLerp" => Raylib_QuaternionLerp,
    "raylib::QuaternionNlerp" => Raylib_QuaternionNlerp,
    "QuaternionNlerp" => Raylib_QuaternionNlerp,
    "raylib::QuaternionSlerp" => Raylib_QuaternionSlerp,
    "QuaternionSlerp" => Raylib_QuaternionSlerp,
    "raylib::QuaternionCubicHermiteSpline" => Raylib_QuaternionCubicHermiteSpline,
    "QuaternionCubicHermiteSpline" => Raylib_QuaternionCubicHermiteSpline,
    "raylib::QuaternionFromVector3ToVector3" => Raylib_QuaternionFromVector3ToVector3,
    "QuaternionFromVector3ToVector3" => Raylib_QuaternionFromVector3ToVector3,
    "raylib::QuaternionFromMatrix" => Raylib_QuaternionFromMatrix,
    "QuaternionFromMatrix" => Raylib_QuaternionFromMatrix,
    "raylib::QuaternionToMatrix" => Raylib_QuaternionToMatrix,
    "QuaternionToMatrix" => Raylib_QuaternionToMatrix,
    "raylib::QuaternionFromAxisAngle" => Raylib_QuaternionFromAxisAngle,
    "QuaternionFromAxisAngle" => Raylib_QuaternionFromAxisAngle,
    "raylib::QuaternionToAxisAngle" => Raylib_QuaternionToAxisAngle,
    "QuaternionToAxisAngle" => Raylib_QuaternionToAxisAngle,
    "raylib::QuaternionFromEuler" => Raylib_QuaternionFromEuler,
    "QuaternionFromEuler" => Raylib_QuaternionFromEuler,
    "raylib::QuaternionToEuler" => Raylib_QuaternionToEuler,
    "QuaternionToEuler" => Raylib_QuaternionToEuler,
    "raylib::QuaternionTransform" => Raylib_QuaternionTransform,
    "QuaternionTransform" => Raylib_QuaternionTransform,
    "raylib::QuaternionEquals" => Raylib_QuaternionEquals,
    "QuaternionEquals" => Raylib_QuaternionEquals,
    "raylib::MatrixCompose" => Raylib_MatrixCompose,
    "MatrixCompose" => Raylib_MatrixCompose,
    "raylib::MatrixDecompose" => Raylib_MatrixDecompose,
    "MatrixDecompose" => Raylib_MatrixDecompose,
}

