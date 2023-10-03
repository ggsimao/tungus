#version 330 core
in vec3 vertexColor;
in vec2 texCoord;

uniform sampler2D ourTexture1, ourTexture2;
uniform float mixer = 0.2;

out vec4 FragColor;

void main()
{
    vec2 newCoords = texCoord;
    newCoords.x = -newCoords.x;
    vec4 texture1 = texture(ourTexture1, texCoord);
    vec4 texture2 = texture(ourTexture2, newCoords);
    FragColor = mix(texture1, texture2, mixer);
}