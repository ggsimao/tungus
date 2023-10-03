#version 330 core
out vec4 fragColor;

uniform vec4 ourColor = vec4(0.0, 0.0, 0.0, 1.0);

void main() {
    fragColor = vec4(ourColor);
}