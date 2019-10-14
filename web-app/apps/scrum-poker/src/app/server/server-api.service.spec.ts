import { TestBed } from "@angular/core/testing";
import {
    HttpClientTestingModule,
    HttpTestingController
} from "@angular/common/http/testing";

import { ServerApiService, ServerApiError } from "./server-api.service";
import { ERROR_CODE } from "../../types";

describe("ServerApiService", () => {
    let httpMock: HttpTestingController;
    beforeEach(() => {
        TestBed.configureTestingModule({
            imports: [HttpClientTestingModule]
        });
        httpMock = TestBed.get(HttpTestingController);
    });

    afterEach(() => {
        httpMock.verify();
    });

    it("should be created", () => {
        const service: ServerApiService = TestBed.get(ServerApiService);
        expect(service).toBeTruthy();
    });

    describe("login", () => {
        it("streams should throws ServerApiError of LOGIN_UNAUTHORIZED when server respond 403", done => {
            const service: ServerApiService = TestBed.get(ServerApiService);

            const authProviderId = "Google";
            const state = {
                authCode: "0123456789"
            };
            service.login(authProviderId, state).subscribe(
                _ => done.fail("Should not produce value"),
                err => {
                    expect(err).toBeInstanceOf(ServerApiError);
                    expect(err.errorCode).toEqual(
                        ERROR_CODE.LOGIN_UNAUTHORIZED
                    );

                    done();
                }
            );

            const req = httpMock.expectOne("api/login");
            expect(req.request.method).toBe("POST");
            expect(req.request.body).toEqual({
                authProviderId,
                state
            });
            req.error(null, {
                status: 401,
                statusText: "Unauthorized"
            });
        });

        it("streams should throws ServerApiError of UNKNOWN_ERROR when server respond non-403 error code", done => {
            const service: ServerApiService = TestBed.get(ServerApiService);

            const authProviderId = "Google";
            const state = {
                authCode: "0123456789"
            };
            service.login(authProviderId, state).subscribe(
                _ => done.fail("Should not produce value"),
                err => {
                    expect(err).toBeInstanceOf(ServerApiError);
                    expect(err.errorCode).toEqual(ERROR_CODE.UNKNOWN_ERROR);

                    done();
                }
            );

            const req = httpMock.expectOne("api/login");
            expect(req.request.method).toBe("POST");
            expect(req.request.body).toEqual({
                authProviderId,
                state
            });
            req.error(null, {
                status: 500,
                statusText: "Internal server error"
            });
        });
    });

    describe("loginStatus", () => {
        it("streams should throws ServerApiError of LOGIN_UNAUTHORIZED when server respond 403", done => {
            const service: ServerApiService = TestBed.get(ServerApiService);

            const accessToken = "access-token";
            service.getLoginStatus(accessToken).subscribe(
                _ => done.fail("Should not produce value"),
                err => {
                    expect(err).toBeInstanceOf(ServerApiError);
                    expect(err.errorCode).toEqual(
                        ERROR_CODE.LOGIN_UNAUTHORIZED
                    );

                    done();
                }
            );

            const req = httpMock.expectOne("api/login/status");
            expect(req.request.method).toBe("GET");
            expect(req.request.headers.get("X-Api-TOken")).toEqual(accessToken);
            req.error(null, {
                status: 401,
                statusText: "Unauthorized"
            });
        });

        it("streams should throws ServerApiError of UNKNOWN_ERROR when server respond non-403 error code", done => {
            const service: ServerApiService = TestBed.get(ServerApiService);

            const accessToken = "access-token";
            service.getLoginStatus(accessToken).subscribe(
                _ => done.fail("Should not produce value"),
                err => {
                    expect(err).toBeInstanceOf(ServerApiError);
                    expect(err.errorCode).toEqual(ERROR_CODE.UNKNOWN_ERROR);

                    done();
                }
            );

            const req = httpMock.expectOne("api/login/status");
            expect(req.request.method).toBe("GET");
            expect(req.request.headers.get("X-Api-TOken")).toEqual(accessToken);
            req.error(null, {
                status: 500,
                statusText: "Internal server error"
            });
        });
    });
});
