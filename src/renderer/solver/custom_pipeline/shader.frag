#version 460

// in vec4 gl_FragCoord;
// in vec2 gl_SamplePosition;

layout ( location = 0 ) flat in int radius;
layout ( location = 1 ) flat in ivec2 color;
layout ( location = 2 ) flat in ivec2 position_entity;

layout ( binding = 0 ) uniform UniformStruct {
    int camera_x;
    int camera_y;
    float camera_size; //TODO: problem with precision
    float window_width;
    float window_height;
    int ssaa_factor;
} uni;

layout ( binding = 1 ) uniform UniformStruct2 {
    int renderer_mode;
    int step;
    float interpolation_ratio;
} uni2;

layout ( location = 0 ) out vec4 FragColor;


void classic_render(vec3 color_final, vec3 color_final_ext, float distance, float radius) {
    if (distance > radius) {
        if ((2 * (float(radius) / uni.camera_size)) > 1 ) {
            discard;
        }
        //discard;
    }

    if (distance < 0.90 * radius) {
        FragColor = vec4(color_final.x, color_final.y, color_final.z, 1.0);
    } else {
        FragColor = vec4(color_final_ext.x, color_final_ext.y, color_final_ext.z, 1.0);
    }
}

void antialiased_render(vec3 color_final, vec3 color_final_ext, float distance, float radius) {
    float alias_ratio = (uni.camera_size / radius) / float(uni.ssaa_factor);
    float ratio_distance = distance / radius;
    float distance_exterior = 1.0 - ratio_distance;
    float alpha_ratio = distance_exterior + alias_ratio / 2;
    float alpha = alpha_ratio * (1 / alias_ratio);
    if (alpha > 1.0) { alpha = 1; }
    if (alpha < 0.0) { alpha = 0; }
    float final_alpha = alpha;
    final_alpha = min(final_alpha, (1 / (alias_ratio)));
    
    if ((final_alpha < 1.0) && (uni2.step == 1)) {
        discard;
    }

    if (final_alpha == 0.0) {
        discard;
    }
    {
        float distance_exterior_color = 0.9 - ratio_distance;
        float ratio_distance_exterior_color = distance_exterior_color + alias_ratio / 2;
        float color_ratio = ratio_distance_exterior_color * (1 / alias_ratio);
        if (color_ratio > 1.0) { color_ratio = 1; }
        if (color_ratio < 0.0) { color_ratio = 0; }

        FragColor = vec4(
            (color_final.x * color_ratio + color_final_ext.x * (1 - color_ratio)),
            (color_final.y * color_ratio + color_final_ext.y * (1 - color_ratio)),
            (color_final.z * color_ratio + color_final_ext.z * (1 - color_ratio)),
            final_alpha
        );
    }
    /*
    if (distance < 0.9 * radius) {
        float ratio = distance / radius;
        float ratio_2 = ratio - (0.9 - alias_ratio / 2);
        if (ratio_2 < 0) { ratio_2 = 0; }

        ratio = 1.0 - ratio_2 * (1 / (alias_ratio));

        if ((final_alpha < 1.0) && (uni2.step == 1)) {
            discard;
        }
        FragColor = vec4(
            (color_final.x * ratio + color_final_ext.x * (1 - ratio)),
            (color_final.y * ratio + color_final_ext.y * (1 - ratio)),
            (color_final.z * ratio + color_final_ext.z * (1 - ratio)),
            final_alpha);

    } else if (distance < 1.0 * radius) {
        float ratio = distance / radius;
        float ratio_2 = (0.9 + alias_ratio / 2) - ratio;
        if (ratio_2 < 0) { ratio_2 = 0; }

        ratio = 1.0 - ratio_2 * (1 / (alias_ratio));

        if ((final_alpha < 1.0) && (uni2.step == 1)) {
            discard;
        }

        FragColor = vec4(
            (color_final_ext.x * ratio + color_final.x * (1 - ratio)),
            (color_final_ext.y * ratio + color_final.y * (1 - ratio)),
            (color_final_ext.z * ratio + color_final.z * (1 - ratio)),
            final_alpha);
    } else {
        if ((final_alpha < 1.0) && (uni2.step == 1)) {
            discard;
        }

        if (final_alpha == 0.0) {
            discard;
        }

        FragColor = vec4(
            (color_final_ext.x),
            (color_final_ext.y),
            (color_final_ext.z),
            final_alpha);
    }
    */
}

void small_radius_experiment(vec3 color_final, vec3 color_final_ext) {
    float pixel_radius = 2 * (float(radius) / uni.camera_size);
    if (pixel_radius < 2) {
        if (pixel_radius > 1) {
            FragColor = vec4(
                color_final.x * 0.9 + color_final_ext.x * 0.1,
                color_final.y * 0.9 + color_final_ext.y * 0.1,
                color_final.z * 0.9 + color_final_ext.z * 0.1,
                min(pixel_radius, 1.0)
            );
        } else {
            FragColor = vec4(
                color_final.x * 0.9 + color_final_ext.x * 0.1,
                color_final.y * 0.9 + color_final_ext.y * 0.1,
                color_final.z * 0.9 + color_final_ext.z * 0.1,
                min(pow(pixel_radius, 2), 1.0)
            );
        }
    }
}
void main() {
    vec2 position_screen = vec2(gl_FragCoord.x / uni.ssaa_factor - uni.window_width / 2, gl_FragCoord.y / uni.ssaa_factor - uni.window_height / 2);
    vec2 position_screen_2 = vec2(position_screen * uni.camera_size);

    ivec2 actual_position = ivec2(int(position_screen_2.x) + uni.camera_x, int(position_screen_2.y) + uni.camera_y);
    ivec2 difference = ivec2(actual_position.x - position_entity.x, actual_position.y - position_entity.y);
    vec2 difference_float = vec2(difference);
    float distance = sqrt(difference_float.x * difference_float.x + difference_float.y * difference_float.y);
    
    /*
    if (radius * 2 * uni.ssaa_factor < uni.camera_size) {
        discard;
    }
    */
    
    int red = 0xFF000000;
    int green = 0x00FF0000;
    int blue = 0x0000FF00;
    
    int and = 0x00000FF;

    vec3 color_final = vec3((color.x >> 0) & and, (color.x >> 8) & and, (color.x >> 16) & and) / 255;
    vec3 color_final_ext = vec3((color.y >> 0) & and, (color.y >> 8) & and, (color.y >> 16) & and) / 255;

    if (uni2.renderer_mode == 1) {
        classic_render(color_final, color_final_ext, distance, radius);
    } else if (uni2.renderer_mode == 2) {
        antialiased_render(color_final, color_final_ext, distance, radius);
    }

    small_radius_experiment(color_final, color_final_ext);
}
