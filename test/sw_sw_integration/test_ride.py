# SPDX-License-Identifier: MPL-2.0
#   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import pytest

from client.services.Ride_service import *
from server_fixtures import *


@pytest.fixture
def sample_rides():
    return [
        Ride(
            journey_departure="2025-03-01T15:15:00Z",
            location_from="Berlin",
            location_to="Hamburg",
            is_template=False,
        ),
        Ride(
            journey_departure="2025-03-02T08:43:00Z",
            journey_arrival="2025-03-02T11:59:00Z",
            location_from="Leipzig",
            location_to="Frankfurt",
            is_template=True,
        ),
    ]


def test_list(api_config_read):
    rides = routes_ride_list(api_config_read)
    assert len(rides) == 0


def test_create(api_config_dict, sample_rides):
    created_ride = routes_ride_post(sample_rides[0], api_config_dict["read_write"])
    assert created_ride.id == 1
    assert created_ride.journey_departure == sample_rides[0].journey_departure
    assert created_ride.location_from == sample_rides[0].location_from
    assert created_ride.location_to == sample_rides[0].location_to
    assert created_ride.is_template == sample_rides[0].is_template

    rides = routes_ride_list(api_config_dict["read"])
    assert len(rides) == 1

    assert rides[0].id == 1
    assert rides[0].journey_departure == sample_rides[0].journey_departure
    assert rides[0].location_from == sample_rides[0].location_from
    assert rides[0].location_to == sample_rides[0].location_to
    assert rides[0].is_template == sample_rides[0].is_template


def test_read(api_config_dict, sample_rides):
    created_ride = routes_ride_post(sample_rides[0], api_config_dict["read_write"])
    assert created_ride.id == 1
    assert created_ride.journey_departure == sample_rides[0].journey_departure
    assert created_ride.location_from == sample_rides[0].location_from
    assert created_ride.location_to == sample_rides[0].location_to
    assert created_ride.is_template == sample_rides[0].is_template

    read_ride = routes_ride_get(1, api_config_dict["read"])
    assert read_ride.id == 1
    assert read_ride.journey_departure == sample_rides[0].journey_departure
    assert read_ride.location_from == sample_rides[0].location_from
    assert read_ride.location_to == sample_rides[0].location_to
    assert read_ride.is_template == sample_rides[0].is_template


def test_update(api_config_dict, sample_rides):
    created_ride = routes_ride_post(sample_rides[0], api_config_dict["read_write"])
    assert created_ride.id == 1

    routes_ride_put(1, sample_rides[1], api_config_dict["read_write"])

    rides = routes_ride_list(api_config_dict["read"])
    assert len(rides) == 1

    assert rides[0].id == 1
    assert rides[0].journey_departure == sample_rides[1].journey_departure
    assert rides[0].journey_arrival == sample_rides[1].journey_arrival
    assert rides[0].location_from == sample_rides[1].location_from
    assert rides[0].location_to == sample_rides[1].location_to
    assert rides[0].is_template == sample_rides[1].is_template


def test_delete(api_config_dict, sample_rides):
    created_ride = routes_ride_post(sample_rides[0], api_config_dict["read_write"])
    assert created_ride.id == 1
    created_ride = routes_ride_post(sample_rides[1], api_config_dict["read_write"])
    assert created_ride.id == 2

    routes_ride_delete(1, api_config_dict["read_write"])

    with pytest.raises(HTTPException) as exc:
        routes_ride_get(1, api_config_dict["read"])
    assert exc.value.status_code == 404

    _ = routes_ride_get(2, api_config_dict["read"])

    rides = routes_ride_list(api_config_dict["read"])
    assert len(rides) == 1

    assert rides[0].id == 2
    assert rides[0].journey_departure == sample_rides[1].journey_departure
    assert rides[0].journey_arrival == sample_rides[1].journey_arrival
    assert rides[0].location_from == sample_rides[1].location_from
    assert rides[0].location_to == sample_rides[1].location_to
    assert rides[0].is_template == sample_rides[1].is_template


def test_list_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_ride_list(api_config_unauthorized)
    assert exc.value.status_code == 401


def test_read_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_ride_get(1, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_create_unauthorized(api_config_unauthorized, sample_rides):
    with pytest.raises(HTTPException) as exc:
        routes_ride_post(sample_rides[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_update_unauthorized(api_config_unauthorized, sample_rides):
    with pytest.raises(HTTPException) as exc:
        routes_ride_put(1, sample_rides[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_delete_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_ride_delete(1, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_create_no_rights(api_config_read, sample_rides):
    with pytest.raises(HTTPException) as exc:
        routes_ride_post(sample_rides[0], api_config_read)
    assert exc.value.status_code == 401


def test_update_no_rights(api_config_read, sample_rides):
    with pytest.raises(HTTPException) as exc:
        routes_ride_put(1, sample_rides[0], api_config_read)
    assert exc.value.status_code == 401


def test_delete_no_rights(api_config_read):
    with pytest.raises(HTTPException) as exc:
        routes_ride_delete(1, api_config_read)
    assert exc.value.status_code == 401


@pytest.fixture
def wrong_owner(api_config_dict, sample_rides):
    _ = routes_ride_post(sample_rides[0], api_config_dict["read_write"])
    return api_config_dict["read_write_2"]


def test_read_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_ride_get(1, wrong_owner)
    assert exc.value.status_code == 404


def test_update_wrong_owner(wrong_owner, sample_rides):
    with pytest.raises(HTTPException) as exc:
        routes_ride_put(1, sample_rides[0], wrong_owner)
    assert exc.value.status_code == 404


def test_delete_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_ride_delete(1, wrong_owner)
    assert exc.value.status_code == 404
