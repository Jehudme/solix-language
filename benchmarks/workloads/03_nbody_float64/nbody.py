if __name__ == "__main__":
    x = 0.5
    y = 1.2
    vx = 0.01
    vy = 0.02

    for _ in range(2000000):
        dist_sq = x * x + y * y + 0.1
        force = 1.0 / dist_sq
        vx += force * 0.001
        vy += force * 0.001
        x += vx
        y += vy

    energy = int(x * 100.0) + int(y * 100.0)
    if energy == 14340304:
        print("RESULT: 14340304")
    else:
        print("FAIL")
