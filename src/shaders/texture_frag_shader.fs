#version 330 core
in vec3 vertexColor;
in vec2 texCoord;

uniform sampler2D ourTexture;

out vec4 FragColor;

void main()
{
    FragColor = texture(ourTexture, texCoord) * vec4(vertexColor, 1.0);
}