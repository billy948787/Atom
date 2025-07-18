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

struct Light{
    vec3 position;
    vec3 direction;
    vec3 color;
    float intensity;
    uint light_type;// 0: point, 1: directional
};

layout(set=0,binding=0)uniform CameraUbo{
    mat4 view;
    mat4 proj;
}camera;

layout(set=0,binding=2)buffer MaterialBuffer{
    Material materials[];
}material_buffer;

layout(set=0,binding=4)buffer LightBuffer{
    Light lights[];
}light_buffer;

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
    
    vec3 ambient_light=vec3(.1,.1,.1);// default ambient light
    
    vec3 total_color=vec3(0.);
    
    for(int i=0;i<light_buffer.lights.length();i++){
        Light light=light_buffer.lights[i];
        vec3 light_dir;
        float intensity=light.intensity;
        
        if(light.light_type==0){// point light
            // transform light position to view space
            vec4 light_pos_view=camera.view*vec4(light.position,1.);
            light_dir=normalize(light_pos_view.xyz-v_position);
            float attenuation=1./(1.+.09*length(light_pos_view.xyz-v_position)+.032*length(light_pos_view.xyz-v_position)*length(light_pos_view.xyz-v_position));
            intensity*=attenuation;
        }else{// directional light
            light_dir=-normalize(light.direction);
        }
        
        vec3 diffuse=diffuse_color(light.color,mat.diffuse_color,v_normal,light_dir);
        vec3 view_dir=normalize(-v_position);// assuming camera is at origin
        
        vec3 specular=specular_color(light.color,mat.specular_color,view_dir,light_dir,v_normal,mat.specular_exponent);
        
        total_color+=(diffuse+specular)*intensity;
    }
    
    vec3 ambient=ambient_color(ambient_light,mat.ambient_color);
    total_color+=ambient;
    
    color=vec4(total_color,1.);
}