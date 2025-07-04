#version 450
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;
            layout(location = 2) in vec2 tex_coord;

            layout(location = 0) out vec3 v_position;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_tex_coord;
            
            void main(){
                gl_Position = vec4(position, 1.0);
                v_position = position;
    v_normal = normal;
    v_tex_coord = tex_coord;
            }