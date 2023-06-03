#version 460

layout ( location = 0 ) out vec3 texture_coords_out;

vec3 vertex_coords[6] = vec3[6](
    vec3(-1.0, 1.0, 0.5),
    vec3(1.0, 1.0, 0.5),
    vec3(-1.0, -1.0, 0.5),
    vec3(-1.0, -1.0, 0.5),
    vec3(1.0, 1.0, 0.5),
    vec3(1.0, -1.0, 0.5)
);

vec3 texture_coords[6] = vec3[6](
    vec3(0.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 1.0, 0.0)
);


void main() {
    gl_Position = vec4(vertex_coords[gl_VertexIndex], 1.0);

    texture_coords_out = texture_coords[gl_VertexIndex];
}