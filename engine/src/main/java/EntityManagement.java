import org.joml.Vector3d;

public class EntityManagement {
    public EntityManagement(ComponentScheduler componentScheduler) {
        Player player = new Player(11);
        componentScheduler.positionComponent.put(11, new Vector3d(0));
        componentScheduler.healtComponent.put(11, 10.0);
    }
}