public class Loop {
    public static void main(String[] arguments) {
        int total = 0;
        for (int index = 0; index < 8000000; index++) {
            total = (total + (index ^ (index % 7))) & 0x7FFFFFFF;
        }

        if (total == 342161131) {
            System.out.println("RESULT: 342161131");
        } else {
            System.out.println("FAIL");
        }
    }
}
