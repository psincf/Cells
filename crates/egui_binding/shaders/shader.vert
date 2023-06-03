#version 460

layout ( location = 0 ) in vec2 pos;
layout ( location = 1 ) in vec2 texture;
layout ( location = 2 ) in int color;

layout ( binding = 0 ) uniform UniformStruct {
    float width;
    float height;
    vec2 rectangle_min;
    vec2 rectangle_max;
} uni;

layout ( location = 0 ) out vec2 texture_coords;
layout ( location = 1 ) out vec4 out_color;

void main() {
    /*
    vec2 position_world_2d = vertex.xy;
    vec2 position_camera_2d = (position_world_2d - vec2(uni.camera_x, uni.camera_y)) * uni.camera_z;
    vec2 position_vertex = position_camera_2d / vec2(uni.window_width / 2, -uni.window_height / 2);
    */

    vec2 pos_2 = vec2((pos.x - uni.width / 2) / (uni.width / 2), -(pos.y - uni.height / 2) / (uni.height / 2));
    gl_Position = vec4(pos_2, 1.0, 1.0);
    texture_coords = texture;

    
    int and = 0x00000FF;

    vec4 color_final = vec4((color >> 0) & and, (color >> 8) & and, (color >> 16) & and, (color >> 24) & and) / 255;
    out_color = vec4(color_final);

}