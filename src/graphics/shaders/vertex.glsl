#version 460
#extension GL_ARB_shader_draw_parameters:enable
layout(location=0)in vec3 position;
layout(location=1)in vec3 normal;
layout(location=2)in vec2 tex_coord;

layout(location=0)out vec3 v_position;
layout(location=1)out vec3 v_normal;
layout(location=2)out vec2 v_tex_coord;
layout(location=3)out flat int v_instance_index;

layout(set=0,binding=0)uniform CameraUbo{
    mat4 view;
    mat4 proj;
}camera;

layout(set=0,binding=1)buffer ModelMatrices{
    mat4 model[];
}models;

layout(set=0,binding=3)buffer NormalMatrices{
    mat4 normal[];
}normals;

void main(){
    mat4 model_matrix=models.model[gl_InstanceIndex];
    mat4 normal_matrix=normals.normal[gl_InstanceIndex];
    
    vec4 temp_position=camera.view*model_matrix*vec4(position,1.);
    
    gl_Position=camera.proj*temp_position;
    v_position=temp_position.xyz;
    v_normal=normalize(normal_matrix*vec4(normal,0.)).xyz;
    v_tex_coord=tex_coord;
    v_instance_index=gl_InstanceIndex;
}