// Счётчик — нажимай A чтобы увеличить, B чтобы сбросить
public class Counter {

    public static int count = 0;

    public static void main() {
        display.show("0");

        while (true) {
            if (buttonA.isPressed()) {
                count = count + 1;
                display.show(count);
                basic.pause(300);
            }

            if (buttonB.isPressed()) {
                count = 0;
                display.show("0");
                basic.pause(300);
            }
        }
    }
}
