#version 460

layout ( location = 0 ) in vec3 vertex;
layout ( location = 1 ) in vec3 color;

layout ( binding = 0 ) uniform UniformStruct {
    float camera_x;
    float camera_y;
    float camera_z;
    float window_width;
    float window_height;
} uni;

layout ( location = 0 ) out vec3 out_color;

void main() {
    vec2 position_world_2d = vertex.xy;
    vec2 position_camera_2d = (position_world_2d - vec2(uni.camera_x, uni.camera_y)) * uni.camera_z;
    vec2 position_vertex = position_camera_2d / vec2(uni.window_width / 2, -uni.window_height / 2);
    gl_Position = vec4(position_vertex, vertex.z, 1.0);

    out_color = color;
}