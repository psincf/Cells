#version 460

layout ( location = 0 ) in vec3 texture_coords;
layout ( location = 0 ) out vec4 FragColor;

layout ( binding = 1 ) uniform texture2DArray texture_array;
layout ( binding = 2 ) uniform sampler sampler_text;

void main() {
    FragColor = texture(sampler2DArray(texture_array, sampler_text), vec3(texture_coords.xyz));
    //FragColor = vec4(0.0, 1.0, 0.0, 1.0);
    if (FragColor.w == 0.0) {
        discard;
    }
    //FragColor.w = 0.1;
}
