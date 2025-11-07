use std::fs::read_to_string;
use std::time::{Duration, Instant};

use crate::gl::types::GLint;
use std::{mem};
use crate::config::UiConfig;
use crate::renderer::vframe::ShaderRawPro;
use crate::{display::SizeInfo, gl::{self, types::{GLfloat, GLsizei, GLsizeiptr, GLuint, GLvoid}}};
#[derive(Debug, Clone)]
pub struct MyFramebuffer {
    pub fbo_id: gl::types::GLuint,     // The FBO itself
    pub texture_id: gl::types::GLuint, // The texture we draw *into*
    pub rbo_id: gl::types::GLuint,     // The depth/stencil buffer
    pub width: i32,
    pub height: i32,
    x_pre: f32,
    y_pre: f32,
    last_cursor_change: f32,
    program: Result<ShaderRawPro, String>,
    config: UiConfig,
    start_time: Instant,

    loc : GLint,
    current_loc: GLint,
    resolution_loc: GLint,
    previous_loc: GLint ,
    color_loc: GLint ,
    prev_color_loc: GLint,
    time_loc: GLint,
    change_loc: GLint,
    pub is_cprogram_loaded: bool,


}
impl Drop for MyFramebuffer {
    fn drop(&mut self) {
        // This code automatically runs when the struct is dropped,
        // (like when you replace it during a window resize)
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo_id);
            gl::DeleteTextures(1, &self.texture_id);
            gl::DeleteRenderbuffers(1, &self.rbo_id);
        }
    }
}
// ➡️ This array defines two triangles that form a quad covering the entire [-1.0, 1.0] NDC space.
const FULL_SCREEN_VERTICES: [f32; 24] = [
    // Position (X, Y),   TexCoords (U, V)
    -1.0,  1.0,  0.0, 1.0, // Top-Left (V1)
    -1.0, -1.0,  0.0, 0.0, // Bottom-Left (V2)
     1.0, -1.0,  1.0, 0.0, // Bottom-Right (V3)

    -1.0,  1.0,  0.0, 1.0, // Top-Left (V1) - Reused
     1.0, -1.0,  1.0, 0.0, // Bottom-Right (V3) - Reused
     1.0,  1.0,  1.0, 1.0, // Top-Right (V4)
];
fn update_projection(u_projection: GLint, size: &SizeInfo) {
    let width = size.width();
    let height = size.height();
    let padding_x = size.padding_x();
    let padding_y = size.padding_y();

    // Bounds check.
    if (width as u32) < (2 * padding_x as u32) || (height as u32) < (2 * padding_y as u32) {
        return;
    }

    // Compute scale and offset factors, from pixel to ndc space. Y is inverted.
    //   [0, width - 2 * padding_x] to [-1, 1]
    //   [height - 2 * padding_y, 0] to [-1, 1]
    let scale_x = 2. / (width - 2. * padding_x);
    let scale_y = -2. / (height - 2. * padding_y);
    let offset_x = -1.;
    let offset_y = 1.;

    unsafe {
        gl::Uniform4f(u_projection, offset_x, offset_y, scale_x, scale_y);
    }
}
/// Creates a new Framebuffer Object (FBO) with a texture to draw into.
///
/// `width` and `height` should be the size of your window.
impl MyFramebuffer {
    pub fn create_framebuffer(
        width: i32,
        height: i32,
        config: UiConfig
        // vao: GLuint,
        // vbo: GLuint,
    ) -> Result<Self, String> {
        let mut loc : GLint = -1;
        let mut current_loc: GLint = -1;
        let mut resolution_loc: GLint = -1;
        let mut previous_loc: GLint  = -1;
        let mut color_loc: GLint  = -1;
        let mut prev_color_loc: GLint = -1;
        let mut time_loc: GLint = -1;
        let mut change_loc: GLint = -1;
        let shaders = &config.general.shaders;
        let (mut vertex_src, mut fragment_src) = (String::new(), String::new());
        if shaders.len() == 2 {
            let vpath = shaders[0].clone();
            let fpath = shaders[1].clone();
            if let (Ok(vsrc), Ok(fsrc)) = (read_to_string(vpath), read_to_string(fpath)) {
                vertex_src = vsrc;
                fragment_src = fsrc;
            }
        }
        let program = ShaderRawPro::new(vertex_src, fragment_src);
        let mut is_cprogram_loaded: bool = false;
        unsafe {
            match program {
                Ok(pro) => {
                    loc = gl::GetUniformLocation(pro.id(), "iChannel0\0".as_ptr() as *const i8);
                    current_loc = gl::GetUniformLocation(pro.id(), "iCurrentCursor\0".as_ptr() as *const i8);
                    resolution_loc = gl::GetUniformLocation(pro.id(), "iResolution\0".as_ptr() as *const i8);
                    previous_loc = gl::GetUniformLocation(pro.id(), "iPreviousCursor\0".as_ptr() as *const i8);
                    color_loc = gl::GetUniformLocation(pro.id(), "iCurrentCursorColor\0".as_ptr() as *const i8);
                    prev_color_loc = gl::GetUniformLocation(pro.id(), "iPreviousCursorColor\0".as_ptr() as *const i8);
                    time_loc = gl::GetUniformLocation(pro.id(), "iTime\0".as_ptr() as *const i8);
                    change_loc = gl::GetUniformLocation(pro.id(), "iTimeCursorChange\0".as_ptr() as *const i8);
                    is_cprogram_loaded = true;
                },
                Err(ref e) => {
                    println!("ShaderError: {}", e);
                }
            }


        }

        let (fbo_id, texture_id, rbo_id) = Self::setup_vframe(width, height);
        Ok(MyFramebuffer { fbo_id: fbo_id, texture_id: texture_id, rbo_id: rbo_id , width: width, height: height, last_cursor_change: 0.0, program: program, x_pre: 0.0, y_pre: 0.0, config: config, start_time: Instant::now(), loc: loc, current_loc: current_loc, resolution_loc:resolution_loc, previous_loc: previous_loc, color_loc: color_loc, prev_color_loc: prev_color_loc, time_loc: time_loc, change_loc: change_loc, is_cprogram_loaded: is_cprogram_loaded })
    }
    pub fn setup_vframe(width: i32, height: i32) -> (GLuint, GLuint, GLuint) {
        let (mut fbo_id, mut texture_id, mut rbo_id) = (0, 0, 0);


        unsafe {
            // 1. --- CREATE THE FRAMEBUFFER ---
            gl::GenFramebuffers(1, &mut fbo_id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo_id);

            // 2. --- CREATE THE COLOR TEXTURE ---
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            // Create an *empty* texture. We give it `null` data.
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null() as *const gl::types::GLvoid, // No data
            );

            // Set filters (required!)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Attach the texture to the FBO
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0, // Attach it as color target
                gl::TEXTURE_2D,
                texture_id,
                0, // Mipmap level
            );

            // 3. --- CREATE THE DEPTH/STENCIL RENDERBUFFER ---
            gl::GenRenderbuffers(1, &mut rbo_id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_id);

            // Allocate storage for depth and stencil
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);

            // Attach the renderbuffer to the FBO
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT, // Attach as depth/stencil
                gl::RENDERBUFFER,
                rbo_id,
            );

            // 4. --- CHECK IF IT'S COMPLETE ---
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return (0, 0, 0);
            }

            // 5. --- UNBIND ---
            // Unbind the FBO so we don't accidentally draw to it
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);


        }
        (fbo_id, texture_id, rbo_id)
    }

    pub fn update_render_data(&mut self, size_info: &SizeInfo, cursor_pos_x: f32, cursor_pos_y: f32, vao: GLuint) {
            let frame_start_time = Instant::now();
            // --- Cursor Rendering ---
            let cellw = size_info.cell_width();
            let cellh = size_info.cell_height();
            if cursor_pos_x != self.x_pre || cursor_pos_y != self.y_pre {
                self.last_cursor_change = self.start_time.elapsed().as_secs_f32();
            }
            // println!("prepos: {}x{} | pos {}x{}", self.x_pre, self.y_pre, cursor_pos_x, cursor_pos_y);
            // println!("time {}", Instant::now().elapsed().as_secs_f32());

            unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                    gl::Disable(gl::BLEND);
                    gl::Viewport(0, 0, size_info.width() as i32, size_info.height() as i32);
                    gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    gl::BindVertexArray(vao);
                    if let Ok(pro) = &self.program {
                        gl::UseProgram(pro.id());
                    }
///////////////////////////////// start send uniform
                    gl::Uniform4f(self.current_loc, cursor_pos_x, cursor_pos_y, cellw, cellh);
                    gl::Uniform2f(self.resolution_loc, size_info.width() as f32, size_info.height() as f32);
                    gl::Uniform4f(self.previous_loc, self.x_pre - size_info.cell_width(), self.y_pre, cellw, cellh);
                    gl::Uniform4f(self.color_loc, 0.65, 0.6, 0.7, 1.0);
                    gl::Uniform4f(self.prev_color_loc, 0.65, 0.6, 0.7, 1.0);
                    gl::Uniform1f(self.time_loc, self.start_time.elapsed().as_secs_f32());
                    gl::Uniform1f(self.change_loc, self.last_cursor_change);
///////////////////////////////// end send uniform
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
                    gl::Uniform1i(self.loc, 0); // Tell iChannel0 to read from slot 0
                    // 7. Draw the full-screen quad!
                    gl::DrawArrays(gl::TRIANGLES, 0, 6);

            }
            // Update previous centered values
            self.x_pre = cursor_pos_x;
            self.y_pre = cursor_pos_y;
            // limit fps
            let elapsed_time = frame_start_time.elapsed();
            let target_frame_duration: Duration = Duration::from_millis(1000 / self.config.window.fps as u64);
            if elapsed_time < target_frame_duration {
                // We finished the frame early, so sleep for the remaining time
                let sleep_duration = target_frame_duration - elapsed_time;
                std::thread::sleep(sleep_duration);
            }
    }
    pub fn setup_full_screen_quad() -> (GLuint,GLuint) {
        let (mut vao, mut vbo) = (0, 0);

        unsafe {
            // 1. Generate and Bind VAO
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // 2. Upload Data to VBO
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (FULL_SCREEN_VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                FULL_SCREEN_VERTICES.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW, // Data won't change often
            );

            let stride = (4 * mem::size_of::<GLfloat>()) as GLsizei; // 4 floats (2 position + 2 UV)

            // 3. Configure Vertex Attribute Pointers (Layout must match the shader)

            // ➡️ Position (Layout Location 0 in shader)
            gl::VertexAttribPointer(
                0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null(), // Offset 0
            );
            gl::EnableVertexAttribArray(0);

            // ➡️ TexCoords (Layout Location 1 in shader)
            gl::VertexAttribPointer(
                1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const GLvoid, // Offset 8 bytes
            );
            gl::EnableVertexAttribArray(1);

            // Cleanup
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        (vao, vbo)
    }
    pub fn resize_vframe(
        &mut self,
    new_width: i32,
    new_height: i32,
) {
    unsafe {
        // 1. --- RESIZE THE COLOR TEXTURE ---
        gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

        // ➡️ Re-allocate its storage with the new size (pass NULL for data)
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            new_width,
            new_height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null() as *const gl::types::GLvoid, // No data
        );
        gl::BindTexture(gl::TEXTURE_2D, 0); // Unbind

        // 2. --- RESIZE THE DEPTH/STENCIL RENDERBUFFER ---
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo_id);

        // ➡️ Re-allocate its storage with the new size
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            new_width,
            new_height,
        );
        gl::BindRenderbuffer(gl::FRAMEBUFFER, 0); // Unbind
        self.width = new_width;
        self.height = new_height;
    }
}
    pub fn start(&self) {
        unsafe {
            // ➡️ THIS IS THE COMMAND YOU ARE MISSING
            // Tell OpenGL to stop drawing to the screen and
            // draw to *your* fbo_id instead.
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo_id);
            gl::Viewport(0, 0, self.width as i32, self.height as i32);

        }
    }

    pub fn stop(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };
    }
    pub fn clear(&self) {
        let color = self.config.colors.primary.background;
        let alpha = self.config.window_opacity();
        unsafe {
            gl::ClearColor(
                (f32::from(color.r) / 255.0).min(1.0) * alpha,
                (f32::from(color.g) / 255.0).min(1.0) * alpha,
                (f32::from(color.b) / 255.0).min(1.0) * alpha,
                alpha,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
    pub fn delete_render(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo_id);
            gl::DeleteTextures(1, &self.texture_id);
            gl::DeleteRenderbuffers(1, &self.rbo_id);
        }
    }
}
