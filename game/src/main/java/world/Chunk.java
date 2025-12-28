package world;

import org.joml.Vector3i;

public record Chunk(float[] block) {
    private static final int chunkSize = 32;
    public Chunk() {
        this(new float[(int) Math.pow(Chunk.chunkSize, 3)]);
    }

    public void addBlock(Vector3i coord, float type) {
        int coord1D = coord.y * (chunkSize * chunkSize) + coord.z * chunkSize + coord.x;
        block[coord1D] = type;
    }

    public float getType(Vector3i coord) {
        return block[coord.y * (chunkSize * chunkSize) + coord.z * chunkSize + coord.x];
    }
}