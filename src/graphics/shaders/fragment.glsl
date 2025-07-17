#version 460
#extension GL_ARB_shader_draw_parameters:enable
layout(location=0)out vec4 color;

layout(location=0)in vec3 v_position;
layout(location=1)in vec3 v_normal;
layout(location=2)in vec2 v_tex_coord;
layout(location=3)in flat int v_instance_index;

struct Material{
    vec3 ambient_color;
    vec3 diffuse_color;
    vec3 specular_color;
    float specular_exponent;
};

layout(set=0,binding=0)uniform CameraUbo{
    mat4 view;
    mat4 proj;
}camera;

layout(set=0,binding=2)buffer MaterialBuffer{
    Material materials[];
}material_buffer;

vec3 ambient_color(vec3 ambient_light,vec3 ambient_material){
    return ambient_light*ambient_material;
}

vec3 diffuse_color(vec3 light_color,vec3 diffuse_material,vec3 normal,vec3 light_dir){
    float diff=max(dot(normal,light_dir),0.);
    return light_color*diffuse_material*diff;
}

vec3 specular_color(vec3 light_color,vec3 specular_material,vec3 view_dir,vec3 light_dir,vec3 normal,float specular_exponent){
    // use the Blinn-Phong reflection model
    vec3 half_vector=normalize(light_dir+view_dir);
    float spec=max(dot(normal,half_vector),0.);
    return light_color*specular_material*pow(spec,specular_exponent);
}

void main(){
    Material mat=material_buffer.materials[v_instance_index];
    
    // simple lighting calculation
    vec3 light_dir=vec3(5.,5.,-5.);
    light_dir=normalize(camera.view*vec4(light_dir,0.)).xyz;
    
    // transform light direction to view space
    float diff=dot(v_normal,light_dir);
    vec3 light_color=vec3(1.,1.,1.) * 5;// white light
    
    vec3 ambient=ambient_color(vec3(.2,.2,.2),mat.ambient_color);
    vec3 diffuse=diffuse_color(light_color,mat.diffuse_color,v_normal,light_dir);
    vec3 specular=specular_color(light_color,mat.specular_color,-normalize(v_position),light_dir,v_normal,mat.specular_exponent);
    
    color=vec4(ambient+diffuse+specular,1.);
}