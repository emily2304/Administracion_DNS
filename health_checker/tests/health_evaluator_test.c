#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "health_evaluator.h"
#include "http_check.h"
#include "tcp_check.h"

static int outcomes[3];
static double rtts[3];
static int call_count;

static int next_result(double *rtt_ms) {
    int index = call_count++;
    *rtt_ms = rtts[index];
    return outcomes[index];
}

int check_tcp_run(const HealthTarget *target, double *rtt_ms) {
    (void)target;
    return next_result(rtt_ms);
}

int check_http_run(const HealthTarget *target, double *rtt_ms) {
    (void)target;
    return next_result(rtt_ms);
}

static void test_majority_and_successful_rtt_average(void) {
    const int results[] = {1, 1, 0};
    const double times[] = {20.0, 40.0, 0.0};
    memcpy(outcomes, results, sizeof(outcomes));
    memcpy(rtts, times, sizeof(rtts));
    call_count = 0;

    HealthTarget target = {.check_type = CHECK_TCP, .retries = 3};
    strcpy(target.record_ip_id, "test-target");
    double average = 0.0;

    assert(health_evaluator_run(&target, &average) == 1);
    assert(call_count == 3);
    assert(average == 30.0);
}

static void test_no_majority_is_unhealthy(void) {
    const int results[] = {1, 0, 0};
    const double times[] = {12.0, 0.0, 0.0};
    memcpy(outcomes, results, sizeof(outcomes));
    memcpy(rtts, times, sizeof(rtts));
    call_count = 0;

    HealthTarget target = {.check_type = CHECK_HTTP, .retries = 3};
    strcpy(target.record_ip_id, "test-target");
    double average = 0.0;

    assert(health_evaluator_run(&target, &average) == 0);
    assert(call_count == 3);
    assert(average == 12.0);
}

int main(void) {
    test_majority_and_successful_rtt_average();
    test_no_majority_is_unhealthy();
    puts("Health Checker: 2 pruebas pasaron.");
    return 0;
}
