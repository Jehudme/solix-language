public class Exc {
    static final class CustomException extends Exception {
        public final int error_code;

        public CustomException(int error_code) {
            this.error_code = error_code;
        }
    }

    public static void main(String[] arguments) {
        int caught_count = 0;
        int normal_sum = 0;

        for (int iteration = 0; iteration < 2000000; iteration++) {
            try {
                normal_sum = (normal_sum + (iteration % 10)) & 0x7FFFFFFF;
            } catch (Exception exception) {
                caught_count += 1;
            }
        }

        for (int iteration = 0; iteration < 20000; iteration++) {
            try {
                throw new CustomException(iteration);
            } catch (CustomException exception) {
                caught_count += exception.error_code;
            }
        }

        int checksum = (normal_sum + caught_count) & 0x7FFFFFFF;
        if (checksum == 208990000) {
            System.out.println("RESULT: 208990000");
        } else {
            System.out.println("FAIL");
        }
    }
}
