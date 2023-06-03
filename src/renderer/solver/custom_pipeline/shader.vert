#version 460

layout ( location = 0 ) in ivec2 base_vertex;

layout ( binding = 0 ) uniform UniformStruct {
    int camera_x;
    int camera_y;
    float camera_size;
    float window_width;
    float window_height;
    int ssaa_factor;
} uni;

layout ( binding = 1 ) uniform UniformStruct2 {
    int renderer_mode;
    int step;
    float interpolation_ratio;
} uni2;

layout ( location = 0 ) out int out_radius;
layout ( location = 1 ) out ivec2 out_color;
layout ( location = 2 ) out ivec2 out_position_entity;

struct DrawableEntity {
    uint old_buffer_id;
    uint old_buffer_id_2;
    int unique_id;
    int unique_id_2;
    int lifetime;
    int position_x;
    int position_y;
    float mass;
    int color;
    int color_2;
};

layout ( binding = 2 ) buffer input_entities {    
    DrawableEntity[] entities;
};

layout ( binding = 3 ) buffer input_entities_2 {
    DrawableEntity[] entities_2;
};
/*
ivec2 interpolate_position() {
    if (entities[gl_InstanceIndex].old_buffer_id == 0xffff && entities[gl_InstanceIndex].old_buffer_id_2 == 0xffff) {
        return ivec2(entities_2[gl_InstanceIndex].position_x, entities_2[gl_InstanceIndex].position_y);
    } else {
        return ivec2( // Doesn't work
            entities[gl_InstanceIndex].position_x + int(float(entities_2[gl_InstanceIndex].position_x - entities[gl_InstanceIndex].position_x) * uni2.interpolation_ratio),
            entities[gl_InstanceIndex].position_y + int(float(entities_2[gl_InstanceIndex].position_y - entities[gl_InstanceIndex].position_y) * uni2.interpolation_ratio)
        );
    }
}

float interpolate_mass() {
    return float(
        entities[gl_InstanceIndex].mass + int(float(entities_2[gl_InstanceIndex].mass - entities[gl_InstanceIndex].mass) * uni2.interpolation_ratio)
    );
}
*/
int compute_radius() {
    return int(sqrt(entities[gl_InstanceIndex].mass / 3.14159265358979323846264338327950288));
}

int compute_z() {
    return int((10000000 * log2(entities[gl_InstanceIndex].mass)) / 64.0);
}

ivec3 compute_vertex() {
    ivec2 vertex = base_vertex * compute_radius();
    vertex = vertex + ivec2(entities[gl_InstanceIndex].position_x, entities[gl_InstanceIndex].position_y);
    //vertex = vertex + interpolate_position();
    return ivec3(vertex, compute_z());
}

void small_radius_experiment(int radius) {
    float pixel_radius = 2 * (float(radius) / uni.camera_size);
    float position_x = gl_Position.x * (uni.window_width / 2) + uni.window_width / 2;
    float position_y = (-gl_Position.y) * (uni.window_height / 2) + uni.window_height / 2;

    ivec2 position_world_2d = ivec2(entities[gl_InstanceIndex].position_x, entities[gl_InstanceIndex].position_y);
    ivec2 position_camera_2d_xy = ivec2(position_world_2d - ivec2(uni.camera_x, uni.camera_y));
    vec2 position_camera_2d = vec2(position_camera_2d_xy) / uni.camera_size;
    vec2 position_vertex_xy = position_camera_2d / vec2(uni.window_width / 2, -uni.window_height / 2);
    float position_center_x = position_vertex_xy.x * (uni.window_width / 2) + uni.window_width / 2;
    float position_center_y = -position_vertex_xy.y * (uni.window_height / 2) + uni.window_height / 2;
    vec2 position_center = vec2(position_center_x, position_center_y);
    vec2 center_difference = position_vertex_xy - trunc(position_vertex_xy);
    
    if (pixel_radius < 1) {
        position_x = position_center.x;
        position_y = position_center.y;

        if (base_vertex.x < -0.9) { position_x = trunc(position_x) - 0.1; }
        else { position_x = trunc(position_x) + 1.1; }
        
        if (base_vertex.y < -0.9) { position_y = trunc(position_y) - 0.1; }
        else { position_y = trunc(position_y) + 1.1; }
        /*
        float amount_x = trunc(position_x + pixel_radius) - trunc(position_x) + 0.1;
        float amount_y = trunc(position_y + pixel_radius) - trunc(position_y) + 0.1;
        
        if (base_vertex.x < -0.9) {
            if (center_difference.x < 0.5) {
                if (amount_x < 2 && pixel_radius > 1) {
                    //position_x = position_x - 1;
                }
                
                if (amount_x < 1 && pixel_radius < 1) {
                    position_x = position_x - 1;
                }
                
                if (amount_x > 2 && pixel_radius < 1) {
                    position_x = position_x + 1;
                }
                
                if (amount_x > 3 && pixel_radius > 1) {
                    //position_x = position_x + 1;
                }
            }
            position_x = trunc(position_x) - 0.1;
        } else {
            if (center_difference.x > 0.5) {
            
                if (amount_x < 1 && pixel_radius < 1) {
                    position_x = position_x + 1;
                }
                
                if (amount_x > 2 && pixel_radius < 1) {
                    position_x = position_x - 1;
                }
            }

            position_x = trunc(position_x) + 0.1;
        }

        if (base_vertex.y < -0.9) {
            if (center_difference.y < 0.5) {

                if (amount_y < 2 && pixel_radius > 1) {
                    //position_y = position_y - 1;
                }

                if (amount_y < 1 && pixel_radius < 1) {
                    position_y = position_y - 1;
                }
                
                if (amount_y > 2 && pixel_radius < 1) {
                    position_y = position_y + 1;
                }
                
                if (amount_y > 3 && pixel_radius > 1) {
                    //position_y = position_y + 1;
                }
            }
            position_y = trunc(position_y) - 0.1;
        } else {
            if (center_difference.y > 0.5) {

                if (amount_y < 1 && pixel_radius < 1) {
                    position_y = position_y + 1;
                }
                
                if (amount_y > 2 && pixel_radius < 1) {
                    position_y = position_y - 1;
                }
            }

            position_y = trunc(position_y) + 0.1;
        }
        */
        gl_Position.x = (position_x - uni.window_width / 2) / (uni.window_width / 2);
        gl_Position.y = -(position_y - uni.window_height / 2) / (uni.window_height / 2);
    }
}

void main() {
    ivec3 vertex = compute_vertex();
    ivec2 position_world_2d = vertex.xy;
    ivec2 position_camera_2d_xy = ivec2(position_world_2d - ivec2(uni.camera_x, uni.camera_y));
    vec2 position_camera_2d = vec2(position_camera_2d_xy) / uni.camera_size;
    
    vec2 position_vertex_xy = position_camera_2d / vec2(uni.window_width / 2, -uni.window_height / 2);
    float position_vertex_z = float(vertex.z) / 10000000; // 10 million
    
    gl_Position = vec4(position_vertex_xy, position_vertex_z, 1.0);

    out_radius = compute_radius();
    out_color = ivec2(entities[gl_InstanceIndex].color, entities[gl_InstanceIndex].color_2);
    out_position_entity = ivec2(entities[gl_InstanceIndex].position_x, entities[gl_InstanceIndex].position_y);

    small_radius_experiment(out_radius);
}