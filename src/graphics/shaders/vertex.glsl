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

layout(set=0,binding=1)buffer ModelMatrices{
    mat4 model[];
}models;

void main(){
    mat4 model_matrix=models.model[gl_InstanceIndex];
    gl_Position=camera.proj*camera.view*model_matrix*vec4(position,1.);
    v_position=position;
    v_normal=normal;
    v_tex_coord=tex_coord;
}