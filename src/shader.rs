use gl;
use gl::types::{GLint, GLuint, GLchar};
use std::io::fs::File;
use std::ptr;
use std::string::String;
use std::vec::Vec;

use check_gl;
use mat4::Mat4;
use numvec::Vec3f;

pub struct Shader {
    program : Program,
}

pub struct Uniform {
    id : GLint,
}

impl Shader {
    pub fn new_from_files(vertex_path: &Path, fragment_path: &Path)
            -> Result<Shader, String> {
        Shader::new_from_source(try!(file_contents(vertex_path)).as_slice(),
                                try!(file_contents(fragment_path)).as_slice())
    }

    pub fn new_from_source(vertex_source: &str, fragment_source: &str)
            -> Result<Shader, String> {
        let vertex = try!(VertexShader::compile(vertex_source));
        let fragment = try!(FragmentShader::compile(fragment_source));
        let program = try!(Program::link(vertex, fragment));
        Ok(Shader { program: program })
    }

    pub fn bind(&self) {
        check_gl!(gl::UseProgram(self.program.id));
    }

    pub fn unbind(&self) {
        check_gl!(gl::UseProgram(0));
    }

    pub fn get_uniform(&self, name: &str) -> Option<Uniform> {
        match name.with_c_str(|c_str| {
            check_gl_unsafe!(gl::GetUniformLocation(self.program.id, c_str))
        }) {
            -1 => None,
            id => Some(Uniform{id: id})
        }
    }

    pub fn set_uniform_i32(&self, uniform: Uniform, value: i32) {
        check_gl!(gl::Uniform1i(uniform.id, value));
    }

    pub fn set_uniform_f32(&self, uniform: Uniform, value: f32) {
        check_gl!(gl::Uniform1f(uniform.id, value));
    }

    pub fn set_uniform_vec3f(&self, uniform: Uniform, value: &Vec3f) {
        check_gl_unsafe!(gl::Uniform3fv(uniform.id, 1, &value.x as *const f32));
    }

    pub fn set_uniform_mat4(&self, uniform: Uniform, value: &Mat4) {
        check_gl_unsafe!(gl::UniformMatrix4fv(
            uniform.id, 1, 0u8, value.as_scalar_ptr()));
    }
}

struct VertexShader { id : GLuint }
impl VertexShader {
    fn compile(source: &str) -> Result<VertexShader, String> {
        compile_any(gl::VERTEX_SHADER, source)
            .map(|id| VertexShader{ id: id })
    }
}
impl Drop for VertexShader {
    fn drop(&mut self) { check_gl!(gl::DeleteShader(self.id)); }
}


struct FragmentShader { id : GLuint }
impl FragmentShader {
    fn compile(source: &str) -> Result<FragmentShader, String> {
        compile_any(gl::FRAGMENT_SHADER, source)
            .map(|id| FragmentShader{ id: id })
    }
}
impl Drop for FragmentShader {
    fn drop(&mut self) { check_gl!(gl::DeleteShader(self.id)); }
}


struct Program { id : GLuint }
impl Program {
    fn link(vertex: VertexShader, fragment: FragmentShader)
            -> Result<Program, String> {
        let program = Program{ id: check_gl!(gl::CreateProgram()) };
        check_gl!(gl::AttachShader(program.id, vertex.id));
        check_gl!(gl::AttachShader(program.id, fragment.id));
        check_gl!(gl::LinkProgram(program.id));
        if link_succeeded(program.id) {
            Ok(program)
        } else {
            let log = get_link_log(program.id);
            Err(format!("Shader linking failed:\n{}", log))
        }
    }
}
impl Drop for Program {
    fn drop(&mut self) { gl::DeleteProgram(self.id); }
}


fn file_contents(path: &Path) -> Result<String, String> {
    File::open(path)
    .and_then(|mut file| file.read_to_end())
    .map_err(|e| String::from_str(e.desc))
    .and_then(|buffer| {
        String::from_utf8(buffer).map_err(|_| {
            format!("File at '{}' is not valid UTF-8.", path.display())
        })
    })
}


fn compile_any(shader_type: u32, source: &str) -> Result<GLuint, String> {
    let id = check_gl!(gl::CreateShader(shader_type));
    assert!(id != 0);
    source.with_c_str(|c_str| {
        check_gl_unsafe!(gl::ShaderSource(id, 1, &c_str, ptr::null()));
    });
    check_gl!(gl::CompileShader(id));
    if compilation_succeeded(id) {
        Ok(id)
    } else {
        let log = get_compilation_log(id);;
        check_gl!(gl::DeleteShader(id));
        if shader_type == gl::VERTEX_SHADER {
            Err(format!("Vertex shader compilation failed:\n{}", log))
        } else {
            Err(format!("Fragment shader compilation failed:\n{}", log))
        }
    }
}


fn compilation_succeeded(id: GLuint) -> bool {
    let mut result: GLint = 0;
    check_gl_unsafe!(gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result));
    result != 0
}


fn get_compilation_log(shader_id: GLuint) -> String {
    let mut log_length = 0;
    check_gl_unsafe!(gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH,
                                     &mut log_length));
    assert!(log_length > 0);
    let mut log_buffer = Vec::from_elem(log_length as uint, 0u8);
    let log_buffer_ptr = log_buffer.as_mut_ptr() as *mut gl::types::GLchar;
    check_gl_unsafe!(gl::GetShaderInfoLog(
            shader_id, log_length, ptr::mut_null(), log_buffer_ptr));
    String::from_utf8(log_buffer).unwrap()
}


fn link_succeeded(id: GLuint) -> bool {
    let mut result: GLint = 0;
    check_gl_unsafe!(gl::GetProgramiv(id, gl::LINK_STATUS, &mut result));
    result != 0
}


fn get_link_log(shader_id: GLuint) -> String {
    let mut log_length = 0;
    check_gl_unsafe!(gl::GetProgramiv(shader_id, gl::INFO_LOG_LENGTH,
                                      &mut log_length));
    assert!(log_length > 0);
    let mut log_buffer = Vec::from_elem(log_length as uint, 0u8);
    let log_buffer_ptr = log_buffer.as_mut_ptr() as *mut gl::types::GLchar;
    check_gl_unsafe!(gl::GetProgramInfoLog(
            shader_id, log_length, ptr::mut_null(), log_buffer_ptr));
    String::from_utf8(log_buffer).unwrap()
}
