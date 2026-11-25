public class VDispatch {
    abstract static class Shape {
        public abstract int getArea(int factor);
    }

    static final class Circle extends Shape {
        @Override
        public int getArea(int factor) {
            return factor * 2;
        }
    }

    static final class Square extends Shape {
        @Override
        public int getArea(int factor) {
            return factor * 3;
        }
    }

    static final class Triangle extends Shape {
        @Override
        public int getArea(int factor) {
            return factor * 4;
        }
    }

    public static void main(String[] arguments) {
        Shape[] shapes = new Shape[] { new Circle(), new Square(), new Triangle() };
        int total = 0;
        for (int iteration = 0; iteration < 100000; iteration++) {
            Shape shape = shapes[iteration % 3];
            total = (total + shape.getArea(iteration % 10)) & 0x7FFFFFFF;
        }

        if (total == 1349997) {
            System.out.println("RESULT: 1349997");
        } else {
            System.out.println("FAIL");
        }
    }
}
