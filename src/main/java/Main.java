import com.naef.jnlua.*;
import org.joml.Vector3i;
import world.Chunk;

ExecutorService service = Executors.newSingleThreadExecutor();
void main() {
    service.submit(Teste::render);
    service.shutdown();

    // Créer un chunk
    Chunk chunk = new Chunk();

    // Ajouter quelques blocs
    // Type 1 = "deart" (vert)
    // Type 2 = "stone" (gris)

    // Créer un sol en pierre
    for (int x = 0; x < 10; x++) {
        for (int z = 0; z < 10; z++) {
            chunk.addBlock(new Vector3i(x, 0, z), 2.0f); // Pierre
        }
    }

    // Ajouter quelques blocs de terre
    chunk.addBlock(new Vector3i(2, 1, 2), 1.0f); // Terre
    chunk.addBlock(new Vector3i(3, 1, 3), 1.0f); // Terre
    chunk.addBlock(new Vector3i(4, 1, 4), 1.0f); // Terre

    // Créer une petite structure
    for (int y = 1; y <= 3; y++) {
        chunk.addBlock(new Vector3i(5, y, 5), 2.0f); // Pilier en pierre
    }

    // Envoyer le chunk au moteur de rendu Rust
    Teste.updateChunk(chunk.block());

    // Mettre à jour la position de la caméra
    Teste.updateValue(16.0, 10.0, 16.0);
}