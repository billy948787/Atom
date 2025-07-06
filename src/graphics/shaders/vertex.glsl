#version 450
layout(location=0)in vec3 position;
layout(location=1)in vec3 normal;
layout(location=2)in vec2 tex_coord;

layout(location=0)out vec3 v_position;
layout(location=1)out vec3 v_normal;
layout(location=2)out vec2 v_tex_coord;

layout(set=0,binding=0)uniform CameraUbo{
    mat4 view;
    mat4 proj;
}camera;

layout(push_constant)uniform PushConstants{
mat4 model;}push_constants;

void main(){
    gl_Position=camera.proj*camera.view*push_constants.model*vec4(position,1.);
    v_position=position;
    v_normal=normal;
    v_tex_coord=tex_coord;
}