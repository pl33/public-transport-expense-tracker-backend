from typing import *

from pydantic import BaseModel, Field

from .Value import Value


class RideTagLink(BaseModel):
    """
    None model
        JSON structure

    """

    model_config = {"populate_by_name": True, "validate_assignment": True}

    id: Optional[int] = Field(validation_alias="id", default=None)

    ride_id: Optional[int] = Field(validation_alias="ride_id", default=None)

    tag_id: Optional[int] = Field(validation_alias="tag_id", default=None)

    order: int = Field(validation_alias="order")

    value: Value = Field(validation_alias="value")

    remarks: Optional[str] = Field(validation_alias="remarks", default=None)
