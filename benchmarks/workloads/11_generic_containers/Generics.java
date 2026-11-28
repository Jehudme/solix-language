public class Generics {
    static final class Box<ValueType> {
        private final ValueType stored_value;

        public Box(ValueType stored_value) {
            this.stored_value = stored_value;
        }

        public ValueType getVal() {
            return this.stored_value;
        }
    }

    public static void main(String[] arguments) {
        int total = 0;
        for (int iteration = 0; iteration < 800000; iteration++) {
            Box<Integer> box_one = new Box<>(iteration);
            Box<Integer> box_two = new Box<>(iteration * 4 + 7);
            total = (total + box_one.getVal() + box_two.getVal()) & 0x7FFFFFFF;
        }

        if (total == 128282240) {
            System.out.println("RESULT: 128282240");
        } else {
            System.out.println("FAIL");
        }
    }
}
