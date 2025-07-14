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

vec3 specular_color(vec3 light_color,vec3 specular_material,vec3 view_dir,vec3 light_dir,vec3 normal){
    // use the Blinn-Phong reflection model
    vec3 half_vector=normalize(light_dir+view_dir);
    float spec=max(dot(normal,half_vector),0.);
    return light_color*specular_material*pow(spec,32.);// shininess factor of 32
}

void main(){
    Material mat=material_buffer.materials[v_draw_id];
    
    // simple lighting calculation
    vec3 light_dir=-normalize(vec3(1.,1.,1.));
    light_dir=normalize(mat3(camera.view)*light_dir);// transform light direction to view space
    // transform light direction to view space
    float diff=dot(v_normal,light_dir);
    vec3 light_color=vec3(1.0, 1.0, 1.0);// white light
    
    vec3 ambient=ambient_color(vec3(.2,.2,.2),mat.ambient_color);
    vec3 diffuse=diffuse_color(light_color,mat.diffuse_color,v_normal,light_dir);
    vec3 specular=specular_color(light_color,mat.specular_color,-normalize(v_position),light_dir,v_normal);
    
    color=vec4(ambient+diffuse+specular,1.);
}