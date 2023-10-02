#version 330 core
// in vec3 vertexColor;
out vec4 fragColor;

uniform vec4 ourColor = vec4(0.0, 0.0, 0.0, 1.0); // input variable from vs (same name and type)

void main() {
    fragColor = vec4(ourColor);
}