/* Isolated virtual CAN test fixture. Refuses real interfaces. No vehicle support. */
#include <linux/can.h>
#include <linux/can/raw.h>
#include <net/if.h>
#include <poll.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc != 2 || strncmp(argv[1], "vcan", 4) != 0 || strlen(argv[1]) >= IFNAMSIZ) return 2;
    char sys[256], real[4096];
    snprintf(sys, sizeof(sys), "/sys/class/net/%s", argv[1]);
    if (!realpath(sys, real) || strncmp(real, "/sys/devices/virtual/net/", strlen("/sys/devices/virtual/net/")) != 0) return 3;
    int fd = socket(PF_CAN, SOCK_RAW, CAN_RAW);
    if (fd < 0) return 4;
    struct ifreq req = {0};
    strncpy(req.ifr_name, argv[1], IFNAMSIZ - 1);
    if (ioctl(fd, SIOCGIFINDEX, &req) < 0) return 5;
    struct sockaddr_can addr = {.can_family = AF_CAN, .can_ifindex = req.ifr_ifindex};
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) return 6;
    for (int ticks = 0; ticks < 1000; ticks++) {
        struct pollfd p = {.fd = fd, .events = POLLIN};
        if (poll(&p, 1, 20) <= 0) continue;
        struct can_frame frame = {0};
        if (read(fd, &frame, sizeof(frame)) != sizeof(frame)) continue;
        if (frame.can_id != 0x7DF && frame.can_id != 0x7E0) continue;
        if (frame.can_dlc != 8 || frame.data[0] != 2 || frame.data[1] != 1) return 7;
        uint8_t pid = frame.data[2];
        struct can_frame reply = {.can_id = 0x7E8, .can_dlc = 8};
        if (pid == 0) {
            uint8_t data[] = {6, 0x41, 0, 0, 0x10, 0, 0, 0};
            memcpy(reply.data, data, sizeof(data));
        } else if (pid == 0x0C) {
            uint8_t data[] = {4, 0x41, 0x0C, 0x1F, 0x40, 0, 0, 0};
            memcpy(reply.data, data, sizeof(data));
        } else return 8;
        if (write(fd, &reply, sizeof(reply)) != sizeof(reply)) return 9;
    }
    close(fd);
    return 0;
}
