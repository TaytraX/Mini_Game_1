plugins {
    id("java")
}

group = "core"
version = "1.0-SNAPSHOT"
val jomlVersion = "1.10.8"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(platform("org.junit:junit-bom:5.10.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")

    implementation("org.joml:joml:$jomlVersion")
}

tasks.test {
    useJUnitPlatform()
}