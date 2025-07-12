#version 460
#extension GL_ARB_shader_draw_parameters:enable
layout(location=0)out vec4 color;

layout(location=0)in vec3 v_position;
layout(location=1)in vec3 v_normal;
layout(location=2)in vec2 v_tex_coord;
layout(location=3)in flat int v_draw_id;

struct Material{
    vec3 ambient_color;
    vec3 diffuse_color;
    vec3 specular_color;
};

layout(set=0,binding=2)buffer MaterialBuffer{
    Material materials[];
}material_buffer;

void main(){
    Material mat=material_buffer.materials[v_draw_id];
    
    color=vec4(mat.diffuse_color,1.);
}