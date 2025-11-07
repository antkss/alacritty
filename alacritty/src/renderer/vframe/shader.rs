use std::ffi::CStr;

use crate::{gl::{self, types::{GLchar, GLenum, GLint, GLuint}}, renderer::{shader::ShaderError}};
#[derive(Debug)]
struct ShaderRaw(GLuint);
fn get_shader_info_log(shader: GLuint) -> String {
    // Get expected log length.
    let mut max_length: GLint = 0;
    unsafe {
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut max_length);
    }

    // Read the info log.
    let mut actual_length: GLint = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(max_length as usize);
    unsafe {
        gl::GetShaderInfoLog(shader, max_length, &mut actual_length, buf.as_mut_ptr() as *mut _);
    }

    // Build a string.
    unsafe {
        buf.set_len(actual_length as usize);
    }

    String::from_utf8_lossy(&buf).to_string()
}
fn get_program_info_log(program: GLuint) -> String {
    // Get expected log length.
    let mut max_length: GLint = 0;
    unsafe {
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut max_length);
    }

    // Read the info log.
    let mut actual_length: GLint = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(max_length as usize);
    unsafe {
        gl::GetProgramInfoLog(program, max_length, &mut actual_length, buf.as_mut_ptr() as *mut _);
    }

    // Build a string.
    unsafe {
        buf.set_len(actual_length as usize);
    }

    String::from_utf8_lossy(&buf).to_string()
}
impl ShaderRaw {
    pub fn new(kind: GLenum, source: String) -> Result<Self, String>{
        let mut sources = Vec::<*const GLchar>::with_capacity(3);
        let mut lengths = Vec::<GLint>::with_capacity(3);

        sources.push(source.as_ptr().cast());
        lengths.push(source.len() as GLint);

        let shader = unsafe { Self(gl::CreateShader(kind)) };

        let mut success: GLint = 0;
        unsafe {
            gl::ShaderSource(
                shader.id(),
                lengths.len() as GLint,
                sources.as_ptr().cast(),
                lengths.as_ptr(),
            );
            gl::CompileShader(shader.id());
            gl::GetShaderiv(shader.id(), gl::COMPILE_STATUS, &mut success);
        }

        if success == GLint::from(gl::TRUE) {
            Ok(shader)
        } else {
            Err(ShaderError::Compile(get_shader_info_log(shader.id())).to_string())
        }
    }

    fn id(&self) -> GLuint {
        self.0
    }
}
#[derive(Copy, Clone, Debug)]
pub struct ShaderRawPro(GLuint);
impl ShaderRawPro {
    pub fn new(vertex_src: String, frag_src: String) -> Result<Self, String> {
        let vertex_shader =
            ShaderRaw::new(gl::VERTEX_SHADER, vertex_src)?;
        let fragment_shader =
            ShaderRaw::new(gl::FRAGMENT_SHADER, frag_src)?;

        let program = unsafe { Self(gl::CreateProgram()) };

        let mut success: GLint = 0;
        unsafe {
            gl::AttachShader(program.id(), vertex_shader.id());
            gl::AttachShader(program.id(), fragment_shader.id());
            gl::LinkProgram(program.id());
            gl::GetProgramiv(program.id(), gl::LINK_STATUS, &mut success);
        }

        if success != i32::from(gl::TRUE) {
            return Err(ShaderError::Link(get_program_info_log(program.id())).to_string());
        }
        Ok(program)
    }

    /// Get uniform location by name. Panic if failed.
    pub fn get_uniform_location(&self, name: &'static CStr) -> Result<GLint, String> {
        // This call doesn't require `UseProgram`.
        let ret = unsafe { gl::GetUniformLocation(self.id(), name.as_ptr()) };
        if ret == -1 {
            return Err(ShaderError::Uniform(name).to_string());
        }
        Ok(ret)
    }
    /// Get the shader program id.
    pub fn id(&self) -> GLuint {
        self.0
    }

}
