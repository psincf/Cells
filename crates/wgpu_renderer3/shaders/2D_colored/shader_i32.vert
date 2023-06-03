#version 460

layout ( location = 0 ) in ivec3 vertex;
layout ( location = 1 ) in vec4 color;

layout ( binding = 0 ) uniform UniformStruct {
    int camera_x;
    int camera_y;
    float camera_size;
    float window_width;
    float window_height;
} uni;

layout ( location = 0 ) out vec4 out_color;

void main() {
    ivec2 position_world_2d = vertex.xy;
    ivec2 position_camera_2d_xy = ivec2(position_world_2d - ivec2(uni.camera_x, uni.camera_y));
    vec2 position_camera_2d = vec2(position_camera_2d_xy) / uni.camera_size;
    
    vec2 position_vertex_xy = position_camera_2d / vec2(uni.window_width / 2, -uni.window_height / 2);
    float position_vertex_z = float(vertex.z) / 10000000; // 10 million
    
    gl_Position = vec4(position_vertex_xy, position_vertex_z, 1.0);

    out_color = color;
}