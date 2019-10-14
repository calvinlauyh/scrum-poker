import { TestBed } from "@angular/core/testing";

import { AuthEventService, EventType } from "./auth-event.service";

describe("AuthEventService", () => {
    beforeEach(() => TestBed.configureTestingModule({}));

    it("should be created", () => {
        const service: AuthEventService = TestBed.get(AuthEventService);
        expect(service).toBeTruthy();
    });

    it("should emit Initial event on construct", done => {
        const service: AuthEventService = TestBed.get(AuthEventService);

        service.$event.subscribe(event => {
            expect(event).toEqual({
                type: EventType.Initial
            });

            done();
        });
    });
});
