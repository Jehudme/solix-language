public class FnPtr {
    interface IntFunction {
        int apply(int input_val);
    }

    public static int functionOne(int input_val) {
        return input_val * 2 + 1;
    }

    public static int functionTwo(int input_val) {
        return input_val * 3 - 2;
    }

    public static int functionThree(int input_val) {
        return input_val * 5 + 4;
    }

    public static void main(String[] arguments) {
        IntFunction[] function_array = new IntFunction[] {
            FnPtr::functionOne,
            FnPtr::functionTwo,
            FnPtr::functionThree
        };

        int total = 0;
        for (int iteration = 0; iteration < 100000; iteration++) {
            int index = iteration % 3;
            IntFunction selected_fn = function_array[index];
            total = (total + selected_fn.apply(iteration % 50)) & 0x7FFFFFFF;
        }

        if (total == 8266617) {
            System.out.println("RESULT: 8266617");
        } else {
            System.out.println("FAIL");
        }
    }
}
