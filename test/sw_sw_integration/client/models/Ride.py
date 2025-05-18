from typing import *

from pydantic import BaseModel, Field

from .RideTagLink import RideTagLink


class Ride(BaseModel):
    """
    None model
        JSON structure

    """

    model_config = {"populate_by_name": True, "validate_assignment": True}

    id: Optional[int] = Field(validation_alias="id", default=None)

    journey_departure: str = Field(validation_alias="journey_departure")

    journey_arrival: Optional[str] = Field(validation_alias="journey_arrival", default=None)

    location_from: str = Field(validation_alias="location_from")

    location_to: str = Field(validation_alias="location_to")

    remarks: Optional[str] = Field(validation_alias="remarks", default=None)

    is_template: bool = Field(validation_alias="is_template")

    tags: Optional[List[Optional[RideTagLink]]] = Field(validation_alias="tags", default=None)
