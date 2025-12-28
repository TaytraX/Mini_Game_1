package world;

import org.joml.Vector3i;

import java.util.HashMap;

public record scene(HashMap<Vector3i, Chunk> scene) {
    public Chunk getChunk(Vector3i key) {
        return scene.get(key);
    }

    public void delete(Vector3i key) {
        scene.remove(key);
    }
}