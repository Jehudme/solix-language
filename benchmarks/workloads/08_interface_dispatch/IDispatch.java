public class IDispatch {
    interface Worker {
        int work(int parameter_val);
    }

    static final class WorkerA implements Worker {
        @Override
        public int work(int parameter_val) {
            return parameter_val + 5;
        }
    }

    static final class WorkerB implements Worker {
        @Override
        public int work(int parameter_val) {
            return parameter_val * 3;
        }
    }

    public static void main(String[] arguments) {
        Worker worker_a = new WorkerA();
        Worker worker_b = new WorkerB();
        int total = 0;
        for (int iteration = 0; iteration < 100000; iteration++) {
            Worker active_worker = (iteration % 2 == 0) ? worker_a : worker_b;
            total = (total + active_worker.work(iteration % 10)) & 0x7FFFFFFF;
        }

        if (total == 1200000) {
            System.out.println("RESULT: 1200000");
        } else {
            System.out.println("FAIL");
        }
    }
}
