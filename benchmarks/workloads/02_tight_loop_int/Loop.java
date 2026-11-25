public class Loop {
    public static void main(String[] arguments) {
        int total = 0;
        for (int index = 0; index < 1000000; index++) {
            total = (total + (index ^ (index % 7))) & 0x7FFFFFFF;
        }

        if (total == 1783293643) {
            System.out.println("RESULT: 1783293643");
        } else {
            System.out.println("FAIL");
        }
    }
}
