#version 460

layout ( location = 0 ) in vec2 texture_coords;
layout ( location = 1 ) in vec4 color;

layout ( binding = 0 ) uniform UniformStruct {
    float width;
    float height;
    float rectangle_min_x;
    float rectangle_min_y;
    float rectangle_max_x;
    float rectangle_max_y;
} uni;

layout ( location = 0 ) out vec4 FragColor;

layout ( binding = 1 ) uniform texture2DArray texture_array;
layout ( binding = 2 ) uniform sampler sampler_text;

void main() {
    int and = 0x00000FF;
    FragColor = vec4(color);

    if (texture_coords != vec2(0.0, 0.0)) {
        FragColor = texture(sampler2DArray(texture_array, sampler_text), vec3(texture_coords, 0.0));
        //FragColor = vec4(FragColor * color);
    }
}
