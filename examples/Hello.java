// Пример j2bit — мигает сердечком на дисплее micro:bit
// Компиляция: j2bit compile Hello.java
public class Hello {

    public static void main() {
        while (true) {
            display.show("Hello, micro:bit!");
            basic.pause(1000);
            display.clear();
            basic.pause(500);

            if (buttonA.isPressed()) {
                display.show("A pressed!");
                basic.pause(500);
            }
        }
    }
}
