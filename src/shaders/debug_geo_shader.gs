#version 430 core
layout (triangles) in;
layout (line_strip, max_vertices = 6) out;

in vec3 geo_normal[];

in VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
} gs_in[];

out VERTEX {
    vec3 pos;
    vec3 normal;
    vec2 texCoords;
} gs_out;

layout (std140, binding = 0) uniform Matrices {
    mat4 modelMat;
    mat4 viewMat;
    mat4 projMat;
};

const float MAGNITUDE = 0.1;

void generate_line(int index, vec3 normal) {
    mat4 calcMat = projMat * viewMat;
    gl_Position = calcMat * vec4(gs_in[index].pos, 1.0);
    EmitVertex();

    gl_Position = calcMat * (vec4(gs_in[index].pos, 1.0) + 
                                        normalize(vec4(normal, 0.0)) * MAGNITUDE);
    EmitVertex();
    EndPrimitive();
}

void main() {
    vec4 out_pos[3] = {
        vec4(gs_in[0].pos, 1.0),
        vec4(gs_in[1].pos, 1.0),
        vec4(gs_in[2].pos, 1.0)
    };

    generate_line(0, geo_normal[0]);
    generate_line(1, geo_normal[1]);
    generate_line(2, geo_normal[2]);
}