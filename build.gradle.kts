plugins {
    id("java")
}

group = "core"
version = "1.0-SNAPSHOT"
val lwjglVersion = "3.3.6"
val jomlVersion = "1.10.8"
val lwjglNatives = "natives-windows"

repositories {
    mavenCentral()
}

dependencies {
    implementation(platform("org.lwjgl:lwjgl-bom:$lwjglVersion"))
    implementation("com.fasterxml.jackson.core:jackson-databind:2.17.0")
    implementation ("org.joml:joml:$jomlVersion")

    implementation(project(":renderer"))
    implementation(project(":game"))
    implementation ("com.nebula2d:jnlua:1.0.4")
}

tasks.test {
    useJUnitPlatform()
}