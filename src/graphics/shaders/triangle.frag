#version 450
            layout(location = 0) out vec4 color;

            layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_tex_coord;
            
            void main(){
                color = vec4(v_position, 1.0); // Red color
            }